use std::time::Duration;

use anyhow::Result;
use authentik_common::SETTINGS;
use axum::{
    Router,
    extract::{
        Request, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    response::{IntoResponse, Response},
};
use axum_server::Handle;
use futures::{sink::SinkExt, stream::StreamExt};
use http::uri::Parts;
use hyper::{StatusCode, Uri};
use nix::sys::signal::Signal;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio_tungstenite::tungstenite::{self as ts};

use crate::backend::BackendClient;

mod assets;

#[derive(Clone)]
struct WebState {
    backend_uri: Uri,
    backend_client: BackendClient,
}

fn make_uri_parts(backend_uri: &Uri, req: &Request) -> Parts {
    let mut parts = backend_uri.clone().into_parts();
    parts.path_and_query = req.uri().path_and_query().cloned();
    parts
}

async fn handle_request(state: WebState, mut req: Request) -> Result<Response, StatusCode> {
    *req.uri_mut() =
        Uri::from_parts(make_uri_parts(&state.backend_uri, &req)).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response = state
        .backend_client
        .request(req)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_response();

    response.headers_mut().remove("Server");
    response
        .headers_mut()
        .insert("X-Powered-By", http::header::HeaderValue::from_static("authentik"));

    Ok(response)
}

async fn proxy_ws(backend_uri: Uri, client_socket: WebSocket, req: Request) {
    let uri = {
        let mut parts = make_uri_parts(&backend_uri, &req);
        parts.scheme = Some("ws".try_into().unwrap());
        match Uri::from_parts(parts) {
            Ok(uri) => uri,
            Err(_) => return,
        }
    };

    let backend_socket = match tokio_tungstenite::connect_async(&uri).await {
        Ok((backend_socket, _)) => backend_socket,
        Err(_) => return,
    };

    let (mut client_sender, mut client_receiver) = client_socket.split();
    let (mut backend_sender, mut backend_receiver) = backend_socket.split();

    let client_to_backend = async {
        while let Some(msg) = client_receiver.next().await {
            let msg = match msg {
                Ok(msg) => match msg {
                    ws::Message::Text(text) => ts::Message::Text(text),
                    ws::Message::Binary(binary) => ts::Message::Binary(binary),
                    ws::Message::Ping(ping) => ts::Message::Ping(ping),
                    ws::Message::Pong(pong) => ts::Message::Pong(pong),
                    ws::Message::Close(_) => break,
                },
                Err(_) => break,
            };

            if backend_sender.send(msg).await.is_err() {
                break;
            }
        }
    };

    let backend_to_client = async {
        while let Some(msg) = backend_receiver.next().await {
            let msg = match msg {
                Ok(msg) => {
                    match msg {
                        ts::Message::Text(text) => ws::Message::Text(text),
                        ts::Message::Binary(binary) => ws::Message::Binary(binary),
                        ts::Message::Ping(ping) => ws::Message::Ping(ping),
                        ts::Message::Pong(pong) => ws::Message::Pong(pong),
                        ts::Message::Close(_) => break,
                        // we can ignore `Frame` frames as recommended by the tungstenite maintainers
                        // https://github.com/snapview/tungstenite-rs/issues/268
                        ts::Message::Frame(_) => continue,
                    }
                }
                Err(_) => break,
            };

            if client_sender.send(msg).await.is_err() {
                break;
            }
        }
    };

    tokio::select! {
        _ = client_to_backend => {},
        _ = backend_to_client => {},
    }

    let _ = client_sender.close().await;
    let _ = backend_sender.close().await;
}

async fn handle_ws(backend_uri: Uri, ws: WebSocketUpgrade, req: Request) -> impl IntoResponse {
    ws.on_upgrade(move |socket| proxy_ws(backend_uri, socket, req))
}

async fn proxy_to_backend(
    State(state): State<WebState>,
    ws: Option<WebSocketUpgrade>,
    req: Request,
) -> impl IntoResponse {
    if let Some(ws) = ws {
        handle_ws(state.backend_uri.clone(), ws, req).await.into_response()
    } else {
        handle_request(state, req).await.into_response()
    }
}

async fn signal_handler(handle: Handle, mut handle_rx: Receiver<Signal>, backend_handle_tx: Sender<Signal>) {
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

pub(crate) async fn run(
    backend_uri: Uri,
    handle_rx: Receiver<Signal>,
    backend_handle_tx: Sender<Signal>,
) -> Result<()> {
    let backend_client = BackendClient::new(&backend_uri);
    let app = Router::new()
        .merge(assets::make_router())
        .layer(
            tower_http::trace::TraceLayer::new_for_http()
                .on_response(tower_http::trace::DefaultOnResponse::new().level(tracing::Level::INFO)),
        )
        .layer(sentry::integrations::tower::SentryLayer::new_from_top())
        .fallback(proxy_to_backend)
        .with_state(WebState {
            backend_uri,
            backend_client,
        });
    let handle = Handle::new();
    tokio::spawn(signal_handler(handle.clone(), handle_rx, backend_handle_tx));
    axum_server::bind(SETTINGS.listen.http)
        .handle(handle)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}
