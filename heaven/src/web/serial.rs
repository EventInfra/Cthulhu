use crate::mqtt::MQTTBroadcast;
use crate::web::WebState;
use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Path, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use crate::manager::PortManagerEntry;

pub async fn serial_handler(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
    ws: WebSocketUpgrade,
) -> Response {
    let port = if let Some(v) = state.manager.get_port(&port_label).await {
        v
    } else {
        return (StatusCode::NOT_FOUND, "Port not found").into_response();
    };
    ws.on_upgrade(|ws| serial_handle_socket(ws, state, port))
}

async fn serial_handle_socket(mut socket: WebSocket, state: WebState, port: PortManagerEntry) {
    let mut receiver = state.broadcast.subscribe();

    socket
        .send(Message::Binary(Bytes::from(port.log_buffer)))
        .await
        .unwrap();

    while let Ok(message) = receiver.recv().await {
        match message {
            MQTTBroadcast::SerialData { label, data } => {
                if label == port.label {
                    socket
                        .send(Message::Binary(Bytes::from(data)))
                        .await
                        .unwrap();
                }
            }
            _ => {}
        }
    }
}
