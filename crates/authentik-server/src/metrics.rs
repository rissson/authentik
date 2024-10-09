use std::time::Duration;

use anyhow::Result;
use authentik_common::SETTINGS;
use axum::{
    Router,
    extract::State,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_server::Handle;
use hyper::{StatusCode, Uri};
use nix::sys::signal::Signal;
use tokio::sync::broadcast::Receiver;

async fn handler(State(_backend_uri): State<Option<Uri>>) -> Result<Response, StatusCode> {
    use prometheus::Encoder;

    let encoder = prometheus::TextEncoder::new();
    let mut buffer = Vec::new();
    if let Err(e) = encoder.encode(&prometheus::gather(), &mut buffer) {
        tracing::error!("Could not gather prometheus metrics: {e}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    let res = match String::from_utf8(buffer) {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Could not convert Prometheus metrics to UTF-8: {e}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    // TODO: handle backend_uri

    Ok(res.into_response())
}

async fn signal_handler(handle: Handle, mut handle_rx: Receiver<Signal>) {
    loop {
        match handle_rx.recv().await {
            Ok(signal) => match signal {
                Signal::SIGINT | Signal::SIGQUIT => {
                    // Quick shutdown
                    handle.shutdown();
                    break;
                }
                Signal::SIGTERM => {
                    // Graceful shutdown
                    handle.graceful_shutdown(Some(Duration::from_secs(30)));
                    break;
                }
                _ => {
                    // Signal is not for us
                    continue;
                }
            },
            Err(_) => continue,
        }
    }
}

pub(crate) async fn run(backend_uri: Option<Uri>, handle_rx: Receiver<Signal>) -> Result<()> {
    let app = Router::new().route("/metrics", get(handler)).with_state(backend_uri);
    let handle = Handle::new();
    tokio::spawn(signal_handler(handle.clone(), handle_rx));
    axum_server::bind(SETTINGS.listen.metrics)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
