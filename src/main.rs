use anyhow::Result;
use hyper::Uri;
use hyperlocal::Uri as UnixUri;
use nix::sys::signal::Signal;
use tokio::{
    signal::unix::{SignalKind, signal},
    sync::broadcast,
};
use tracing_subscriber::prelude::*;

use crate::common::{SETTINGS, constants};

mod backend;
mod common;
mod core;
mod crypto;
mod metrics;
mod orm;
mod providers;
mod server;
mod utils;
mod web;

async fn signal_handler(
    axum_handles_tx: broadcast::Sender<Signal>,
    backend_handles_tx: broadcast::Sender<Signal>,
) -> Result<()> {
    let mut signal_hangup = signal(SignalKind::hangup())?;
    let mut signal_interrupt = signal(SignalKind::interrupt())?;
    let mut signal_quit = signal(SignalKind::quit())?;
    let mut signal_terminate = signal(SignalKind::terminate())?;
    let mut signal_user_defined2 = signal(SignalKind::user_defined2())?;

    // If signals are added here, make sure they are handled by the respective components
    loop {
        tokio::select! {
            _ = signal_hangup.recv() => {
                // SIGHUP
                // Gunicorn: reload its conf and restart workers gracefully
                // TODO: reload our configuration as well
                let _ = backend_handles_tx.send(Signal::SIGHUP);
            },
            _ = signal_interrupt.recv() => {
                // SIGINT
                // Gunicorn: quick shutdown
                // Axum: quick shutdown
                let _ = axum_handles_tx.send(Signal::SIGINT);
                let _ = backend_handles_tx.send(Signal::SIGINT);
                break;
            },
            _ = signal_quit.recv() => {
                // SIGQUIT
                // Gunicorn: quick shutdown
                // Axum: quick shutdown
                let _ = axum_handles_tx.send(Signal::SIGQUIT);
                let _ = backend_handles_tx.send(Signal::SIGQUIT);
                break;
            },
            _ = signal_terminate.recv() => {
                // SIGQUIT
                // Axum: graceful shutdown after all requests or timeout
                //       plus triggers the following:
                // Gunicorn: graceful shutdown after all requests or timeout
                let _ = axum_handles_tx.send(Signal::SIGTERM);
                break;
            },
            _ = signal_user_defined2.recv() => {
                // SIGUSR2
                // Gunicorn: graceful restart
                let _ = backend_handles_tx.send(Signal::SIGUSR2);
            },
        };
    }

    Ok(())
}

fn sentry_init() -> Option<sentry::ClientInitGuard> {
    if SETTINGS.error_reporting.enabled {
        let guard = sentry::init((SETTINGS.error_reporting.sentry_dsn.clone(), sentry::ClientOptions {
            release: Some(std::borrow::Cow::Owned(format!("authentik@{}", constants::VERSION))),
            environment: Some(std::borrow::Cow::Owned(SETTINGS.error_reporting.environment.clone())),
            traces_sample_rate: SETTINGS.error_reporting.sample_rate,
            // traces_sampler: TODO: configure this
            attach_stacktrace: true,
            send_default_pii: SETTINGS.error_reporting.send_pii,
            // TODO: make this a helper and support build hash
            user_agent: format!("authentik@{}", constants::VERSION).into(),
            session_mode: sentry::SessionMode::Request,
            default_integrations: true,
            ..Default::default()
        }));
        return Some(guard);
    }
    None
}

fn tracing_init() {
    use tracing_subscriber::{
        filter::LevelFilter,
        fmt::{Layer, time::UtcTime},
        registry::Registry,
    };
    let filter = LevelFilter::from_level(SETTINGS.log_level);
    let registry = Registry::default()
        .with(filter)
        .with(sentry::integrations::tracing::layer());
    let fmt = Layer::default()
        .with_writer(std::io::stderr)
        .with_timer(UtcTime::rfc_3339());
    if SETTINGS.debug {
        let fmt = fmt.compact().with_ansi(true);
        registry.with(fmt).init();
    } else {
        let fmt = fmt
            .json()
            .with_current_span(true)
            .with_span_list(true)
            .flatten_event(true);
        registry.with(fmt).init();
    }
}

fn main() {
    let _guard = sentry_init();
    tracing_init();

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let backend_uri = if SETTINGS.debug {
                "http://localhost:8000".parse::<Uri>().unwrap()
            } else {
                UnixUri::new(format!("{}/authentik-core.sock", std::env::temp_dir().display()), "/").into()
            };

            let (axum_handles_tx, axum_handles_rx) = broadcast::channel(16);
            let (backend_handles_tx, backend_handles_rx) = broadcast::channel(16);

            tokio::try_join!(
                metrics::run(Some(backend_uri.clone()), axum_handles_tx.subscribe()),
                backend::run(backend_uri.clone(), backend_handles_rx),
                web::run_plain(
                    backend_uri.clone(),
                    axum_handles_tx.subscribe(),
                    backend_handles_tx.clone()
                ),
                web::run_tls(backend_uri.clone(), axum_handles_rx),
                signal_handler(axum_handles_tx, backend_handles_tx),
            )
            .unwrap();
        });
}
