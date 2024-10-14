use anyhow::Result;
use axum::{
    extract::{
        Request, State, WebSocketUpgrade,
        ws::{self, WebSocket},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
};
use futures::{sink::SinkExt, stream::StreamExt};
use hyper::Uri;
use tokio_tungstenite::tungstenite::{self as ts};

use crate::{server::utils::state::AppState, utils::make_uri_parts};

async fn handle_request(state: AppState, mut req: Request) -> Result<Response, StatusCode> {
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

fn handle_ws(backend_uri: Uri, ws: WebSocketUpgrade, req: Request) -> impl IntoResponse {
    ws.on_upgrade(move |socket| proxy_ws(backend_uri, socket, req))
}

pub(super) async fn proxy_to_backend(
    State(state): State<AppState>,
    ws: Option<WebSocketUpgrade>,
    req: Request,
) -> impl IntoResponse {
    if let Some(ws) = ws {
        handle_ws(state.backend_uri, ws, req).into_response()
    } else {
        handle_request(state, req).await.into_response()
    }
}
