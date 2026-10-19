//! HTTP server for the editor: static UI and the editor WebSocket.

use std::sync::Arc;

use axum::Router;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State as AxState};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc::unbounded_channel;

use super::hub::{Hub, Msg};
use crate::ui::UiSource;

#[derive(Clone)]
pub struct State {
    pub hub: Arc<Hub>,
    pub ui: UiSource,
    pub token: String,
    pub port: u16,
}

pub async fn serve(listener: tokio::net::TcpListener, state: State) -> std::io::Result<()> {
    let app = Router::new()
        .route("/ws", get(ws))
        .fallback(get(asset))
        .with_state(state);
    axum::serve(listener, app).await
}

/// Only pages served by this server may open the socket; otherwise any site
/// open in another tab could drive the preview.
pub fn origin_ok(headers: &HeaderMap, port: u16) -> bool {
    let Some(origin) = headers.get(header::ORIGIN).and_then(|o| o.to_str().ok()) else {
        return false;
    };
    origin == format!("http://127.0.0.1:{port}") || origin == format!("http://localhost:{port}")
}

#[derive(Deserialize)]
struct WsQuery {
    token: Option<String>,
}

async fn ws(
    upgrade: WebSocketUpgrade,
    headers: HeaderMap,
    Query(q): Query<WsQuery>,
    AxState(state): AxState<State>,
) -> Response {
    if !origin_ok(&headers, state.port) {
        return (StatusCode::FORBIDDEN, "origin not allowed").into_response();
    }
    if q.token.as_deref() != Some(state.token.as_str()) {
        return (
            StatusCode::UNAUTHORIZED,
            "missing or wrong token; open the URL printed by `oxframe dev`",
        )
            .into_response();
    }
    upgrade.on_upgrade(move |socket| editor(socket, state.hub))
}

async fn editor(socket: WebSocket, hub: Arc<Hub>) {
    let (mut sink, mut stream) = socket.split();
    let (tx, mut rx) = unbounded_channel::<Msg>();
    let id = hub.add_ui(tx);
    let writer = tokio::spawn(async move {
        while let Some(m) = rx.recv().await {
            let msg = match m {
                Msg::Text(t) => Message::Text(t.into()),
                Msg::Binary(b) => Message::Binary(b.into()),
            };
            if sink.send(msg).await.is_err() {
                break;
            }
        }
    });
    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(t) => hub.forward_from_ui(Msg::Text(t.to_string())),
            Message::Binary(b) => hub.forward_from_ui(Msg::Binary(b.to_vec())),
            Message::Close(_) => break,
            _ => {}
        }
    }
    writer.abort();
    hub.remove_ui(id);
}

async fn asset(uri: Uri, AxState(state): AxState<State>) -> Response {
    let path = uri.path();
    let (data, mime) = match state.ui.get(path) {
        Some(found) => found,
        // Client-side routes fall back to the app shell.
        None => match state.ui.get("index.html") {
            Some(index) if !path.contains('.') => index,
            _ => return (StatusCode::NOT_FOUND, "not found").into_response(),
        },
    };
    let body = if mime.starts_with("text/html") {
        let html = String::from_utf8_lossy(&data);
        let config = r#"<script>window.__OXFRAME_CONFIG__={"backend":"ws","url":"/ws"}</script>"#;
        html.replacen("</head>", &format!("{config}</head>"), 1)
            .into_bytes()
    } else {
        data.into_owned()
    };
    (
        [
            (header::CONTENT_TYPE, mime),
            (header::CACHE_CONTROL, "no-cache".into()),
        ],
        body,
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_check() {
        let mut h = HeaderMap::new();
        assert!(!origin_ok(&h, 3000), "browsers always send Origin for WS");
        h.insert(header::ORIGIN, "http://127.0.0.1:3000".parse().unwrap());
        assert!(origin_ok(&h, 3000));
        h.insert(header::ORIGIN, "http://localhost:3000".parse().unwrap());
        assert!(origin_ok(&h, 3000));
        h.insert(header::ORIGIN, "http://evil.example".parse().unwrap());
        assert!(!origin_ok(&h, 3000));
        h.insert(header::ORIGIN, "http://127.0.0.1:3001".parse().unwrap());
        assert!(!origin_ok(&h, 3000));
    }
}
