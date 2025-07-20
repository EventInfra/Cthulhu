use crate::web::WebState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use cthulhu_common::status::JobCommand;
use tracing::warn;

pub mod index;
pub mod port;

pub async fn restart_all(State(state): State<WebState>) -> Response {
    match state.mqtt.broadcast_command(JobCommand::RestartAngel).await {
        Ok(_) => {
            Html("OK").into_response()
        }
        Err(e) => {
            warn!("Failed to send restart: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Error").into_response()
        }
    }
}

pub async fn abort(
    State(state): State<WebState>,
    Path(port_label): Path<String>,
) -> impl IntoResponse {
    state
        .mqtt
        .send_command(&port_label, JobCommand::ResetJob)
        .await
        .unwrap();
    Html("DONE".to_string())
}
