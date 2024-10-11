use std::time::Duration;

use axum::extract::Request;
use axum_server::Handle;
use http::uri::Parts;
use hyper::Uri;
use nix::sys::signal::Signal;
use tokio::sync::broadcast::Receiver;

pub(crate) fn make_uri_parts(backend_uri: &Uri, req: &Request) -> Parts {
    let mut parts = backend_uri.clone().into_parts();
    parts.path_and_query = req.uri().path_and_query().cloned();
    parts
}

pub(crate) async fn signal_handler(handle: Handle, mut handle_rx: Receiver<Signal>) {
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
