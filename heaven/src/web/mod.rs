use crate::manager::JobManager;
use crate::mqtt::{BroadcastSender, MQTTSender};
use crate::web::pages::{abort, restart_all};
use crate::web::serial::serial_handler;
use axum::Router;
use axum::body::Body;
use axum::extract::Path;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use cthulhu_config::heaven::HeavenConfig;
use include_dir::{Dir, include_dir};
use tower_http::catch_panic::CatchPanicLayer;
use tracing::info;

mod pages;

mod serial;
mod helpers;

#[derive(Clone)]
struct WebState {
    manager: JobManager,
    mqtt: MQTTSender,
    broadcast: BroadcastSender,
}

pub async fn web_main(
    config: HeavenConfig,
    manager: JobManager,
    mqtt: MQTTSender,
    broadcast: BroadcastSender,
) -> color_eyre::Result<()> {
    let state = WebState {
        manager,
        mqtt,
        broadcast,
    };
    let app = Router::new()
        .route("/", get(pages::index::index))
        .route("/portstatus.html", get(pages::index::port_status))
        .route("/restart", get(restart_all))
        .route("/port/{port_label}/", get(pages::port::port))
        .route("/port/{port_label}/header.html", get(pages::port::header))
        .route("/port/{port_label}/devinfo.html", get(pages::port::footer))
        .route("/port/{port_label}/abort", get(abort))
        .route("/port/{port_label}/serial", get(serial_handler))
        .route("/assets/{*path}", get(static_path))
        .layer(CatchPanicLayer::new())
        .with_state(state);

    info!("Starting web...");
    let listener = tokio::net::TcpListener::bind(&config.web.listen_address).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/web/assets");

async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(Body::from(file.contents()))
            .unwrap(),
    }
}
