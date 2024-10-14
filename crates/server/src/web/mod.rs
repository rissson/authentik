use std::time::Duration;

use anyhow::Result;
use authentik_common::SETTINGS;
use authentik_server_utils::{backend::BackendClient, state::AppState};
use axum::Router;
use axum_server::{Handle, tls_openssl::OpenSSLConfig};
use hyper::Uri;
use nix::sys::signal::Signal;
use sea_orm::Database;
use tokio::sync::broadcast::{Receiver, Sender};

use crate::{
    crypto::{generate_self_signed_cert, get_tls_acceptor_builder},
    utils::signal_handler,
};

mod assets;
mod backend;

async fn signal_handler_with_backend(
    handle: Handle,
    mut handle_rx: Receiver<Signal>,
    backend_handle_tx: Sender<Signal>,
) {
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
                    let _ = backend_handle_tx.send(Signal::SIGTERM);
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

async fn make_router(backend_uri: &Uri) -> Result<Router> {
    let db_url = "postgres://postgres@localhost/authentik";
    let db = Database::connect(db_url).await?;

    let backend_client = BackendClient::new(backend_uri);
    Ok(Router::new()
        .merge(assets::make_router())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(sentry::integrations::tower::SentryLayer::new_from_top())
        .fallback(backend::proxy_to_backend)
        .with_state(AppState {
            backend_uri: backend_uri.clone(),
            backend_client,
            db,
        }))
}

pub(crate) async fn run_plain(
    backend_uri: Uri,
    handle_rx: Receiver<Signal>,
    backend_handle_tx: Sender<Signal>,
) -> Result<()> {
    let router = make_router(&backend_uri).await?;
    let handle = Handle::new();

    tokio::spawn(signal_handler_with_backend(
        handle.clone(),
        handle_rx,
        backend_handle_tx,
    ));

    axum_server::bind(SETTINGS.listen.http)
        .handle(handle)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}

pub(crate) async fn run_tls(backend_uri: Uri, handle_rx: Receiver<Signal>) -> Result<()> {
    let router = make_router(&backend_uri).await?;
    let handle = Handle::new();

    let (cert, pkey) = generate_self_signed_cert()?;
    let acceptor = {
        let mut acceptor = get_tls_acceptor_builder()?;
        acceptor.set_private_key(pkey.as_ref())?;
        acceptor.set_certificate(cert.as_ref())?;
        acceptor.build()
    };
    let openssl_config = OpenSSLConfig::from_acceptor(acceptor.into());

    tokio::spawn(signal_handler(handle.clone(), handle_rx));
    axum_server::bind_openssl(SETTINGS.listen.https, openssl_config)
        .handle(handle)
        .serve(router.into_make_service())
        .await?;
    Ok(())
}
