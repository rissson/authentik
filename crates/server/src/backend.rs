use std::{
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Result, anyhow};
use authentik_common::SETTINGS;
use axum::body::Body;
use http::uri::PathAndQuery;
use hyper::{Method, Request, Uri};
use hyper_util::{
    client::legacy::{Client, ResponseFuture, connect::HttpConnector},
    rt::TokioExecutor,
};
use hyperlocal::UnixConnector;
use nix::{
    sys::signal::{self, Signal},
    unistd::Pid,
};
use tokio::{
    fs,
    sync::broadcast::Receiver,
    time::{Duration, interval},
};

type UnixClient = Client<UnixConnector, Body>;
type HttpClient = Client<HttpConnector, Body>;

#[derive(Clone)]
pub(crate) enum BackendClient {
    Http(HttpClient),
    Unix(UnixClient),
}

impl BackendClient {
    pub(crate) fn new(backend_uri: &Uri) -> Self {
        match backend_uri.scheme_str() {
            Some("unix") => {
                let client = Client::builder(TokioExecutor::new()).build(UnixConnector);
                Self::Unix(client)
            }
            _ => {
                let client = Client::builder(TokioExecutor::new()).build_http();
                Self::Http(client)
            }
        }
    }

    pub(crate) fn request(&self, req: Request<Body>) -> ResponseFuture {
        match self {
            Self::Http(client) => client.request(req),
            Self::Unix(client) => client.request(req),
        }
    }
}

struct Backend {
    command: Command,
    child: Option<Pid>,
    pid_file_path: Option<PathBuf>,
}

impl Backend {
    fn new() -> Result<Self> {
        let pid_file_path = if !SETTINGS.debug {
            let pid_file = tempfile::Builder::new()
                .prefix("authentik-gunicorn.")
                .suffix(".pid")
                .tempfile()?;
            let (_, pid_file_path) = pid_file.keep()?;
            Some(pid_file_path)
        } else {
            None
        };

        let command = if !SETTINGS.debug {
            let mut command = Command::new("gunicorn");
            command.args([
                "-c",
                "./lifecycle/gunicorn.conf.py",
                "authentik.root.asgi:application",
                "--pid",
                &pid_file_path.clone().unwrap().into_os_string().into_string().unwrap(),
            ]);
            command
        } else {
            let mut command = Command::new("./manage.py");
            command.args(["dev_server"]);
            command
        };

        Ok(Backend {
            command,
            child: None,
            pid_file_path,
        })
    }

    fn start(&mut self) -> Result<()> {
        tracing::debug!("starting backend");
        self.child = Some(Pid::from_raw(self.command.spawn()?.id() as i32));
        Ok(())
    }

    fn reload(&self) {
        tracing::debug!("reloading backend");
        match &self.child {
            Some(child) => {
                if let Err(errno) = signal::kill(*child, Signal::SIGHUP) {
                    tracing::warn!("Failed to reload gunicorn: {}", errno);
                }
            }
            None => tracing::warn!("No gunicorn process launched, ignoring"),
        }
    }

    async fn restart(&mut self) {
        tracing::debug!("restarting backend");
        if self.child.is_none() {
            tracing::warn!("No gunicorn process launched, ignoring");
            return;
        }

        let child = self.child.unwrap();

        if let Err(errno) = signal::kill(child, Signal::SIGUSR2) {
            tracing::warn!("Failed to restart gunicorn: {}", errno);
            return;
        }

        let new_pid_file: PathBuf = {
            let mut new_pid_file = self.pid_file_path.clone().unwrap().into_os_string();
            new_pid_file.push(".2");
            new_pid_file.into()
        };

        let mut interval = interval(Duration::from_secs(1));
        loop {
            tracing::debug!(
                "waiting for new gunicorn pidfile to appear at {}",
                &new_pid_file.display()
            );
            interval.tick().await;
            match new_pid_file.as_path().try_exists() {
                Ok(true) => break,
                Ok(false) => continue,
                Err(e) => {
                    tracing::warn!("failed to find the new gunicorn process, aborting: {}", e);
                    return;
                }
            }
        }

        let new_pid_s = match fs::read_to_string(&new_pid_file).await {
            Ok(contents) => contents.trim().to_owned(),
            Err(e) => {
                tracing::warn!("failed to find the new gunicorn process, aborting: {}", e);
                return;
            }
        };
        let new_pid = match new_pid_s.parse::<i32>() {
            Ok(pid) => Pid::from_raw(pid),
            Err(e) => {
                tracing::warn!("failed to find the new gunicorn process, aborting: {}", e);
                return;
            }
        };

        tracing::warn!("new gunicorn PID is {}", &new_pid);
        tracing::warn!("gracefully stopping old gunicorn");
        self.graceful_shutdown();
        self.child = Some(new_pid);
    }

    fn quick_shutdown(&self) {
        tracing::debug!("quickly shutting down backend");
        match &self.child {
            Some(child) => {
                if let Err(errno) = signal::kill(*child, Signal::SIGTERM) {
                    tracing::warn!("Failed to shutdown gunicorn: {}", errno);
                }
            }
            None => tracing::warn!("No gunicorn process launched, ignoring"),
        }
    }

    fn graceful_shutdown(&self) {
        tracing::debug!("gracefully shutting down backend");
        match &self.child {
            Some(child) => {
                if let Err(errno) = signal::kill(*child, Signal::SIGTERM) {
                    tracing::warn!("Failed to shutdown gunicorn: {}", errno);
                }
            }
            None => tracing::warn!("No gunicorn process launched, ignoring"),
        }
    }
}

async fn healthcheck(backend_uri: Uri) -> Result<()> {
    let mut parts = backend_uri.into_parts();
    parts.path_and_query = Some(PathAndQuery::from_static("/-/health/live/"));
    let backend_uri = Uri::from_parts(parts).unwrap();

    let client = BackendClient::new(&backend_uri);

    let req = Request::builder()
        .method(Method::GET)
        .uri(backend_uri)
        .header("Host", "localhost")
        .header("User-Agent", "goauthentik.io/router/healthcheck")
        .body(Body::empty())
        .unwrap();

    let response = client.request(req).await?;

    if response.status().is_success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Healthcheck returned non-2xx status: {}",
            response.status().as_u16()
        ))
    }
}

pub(crate) async fn run(backend_uri: Uri, mut handle_rx: Receiver<Signal>) -> Result<((), ())> {
    let mut backend = Backend::new()?;
    backend.start()?;

    let running = AtomicBool::new(true);

    tokio::try_join!(
        async {
            let max_fails = 100;
            let mut failed_checks = 0;
            let mut interval = interval(Duration::from_secs(1));
            while failed_checks < max_fails && running.load(Ordering::Relaxed) {
                tracing::debug!("waiting for backend to be healthy");
                if let Ok(()) = healthcheck(backend_uri.clone()).await {
                    break;
                }
                failed_checks += 1;
                interval.tick().await;
            }

            if failed_checks >= max_fails {
                return Err(anyhow!("Backend failed to start within 100 seconds"));
            }
            Ok(())
        },
        async {
            loop {
                match handle_rx.recv().await {
                    Ok(signal) => {
                        tracing::debug!("received signal {}", &signal);
                        match signal {
                            Signal::SIGHUP => {
                                backend.reload();
                            }
                            Signal::SIGINT | Signal::SIGQUIT => {
                                backend.quick_shutdown();
                                break;
                            }
                            Signal::SIGTERM => {
                                backend.graceful_shutdown();
                                break;
                            }
                            Signal::SIGUSR2 => {
                                backend.restart().await;
                            }
                            _ => {
                                // Signal is not for us
                                continue;
                            }
                        };
                    }
                    Err(_) => continue,
                }
            }
            running.store(false, Ordering::Relaxed);
            Ok(())
        }
    )
}
