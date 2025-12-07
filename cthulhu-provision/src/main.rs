use std::str::FromStr;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Router;
use axum::routing::get;
use clap::Parser;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{Layer, Registry};
use tracing_subscriber::layer::SubscriberExt;
use cthulhu_config::LoadableConfig;
use cthulhu_config::provision::ProvisionConfig;
use crate::args::Cli;
use crate::state::{AppState, AppStateHandle};

mod args;
mod arista;
mod juniper;
mod state;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    let config = ProvisionConfig::from_file(&cli.config).await?;

    // Initialize logging
    let max_log_level =
        Level::from_str(&(config.log_level.as_ref().unwrap_or(&"info".to_string())))?;
    let stdsub =
        tracing_subscriber::fmt::layer().with_filter(LevelFilter::from_level(max_log_level));
    let subscriber = Registry::default().with(stdsub);
    tracing::subscriber::set_global_default(subscriber)?;

    let state = Arc::new(AppState {
        config_server: config.config_server.clone(),
        os_mappings: config.model_os_mappings.clone(),
        autoreload: config.autoreload_config.clone(),
    });

    let app = Router::new()
        .layer(TraceLayer::new_for_http())
        .nest("/provision/arista", arista::get_script_routes())
        .nest("/provision/juniper", juniper::get_script_routes())
        .route("/configuration/{vendor}/{serial_number}", get(proxy_configuration))
        // `GET /` goes to `root`
        .route("/", get(root))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.web.listen_address.as_str()).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn proxy_configuration(State(state): State<AppStateHandle>, Path((vendor, serial_number)): Path<(String, String)>) -> impl IntoResponse {
    let final_url = state.config_server.replace("%VENDOR%", vendor.as_str()).replace("%SERIAL_NUMBER%", serial_number.as_str());
    let resp = reqwest::get(final_url.as_str()).await.unwrap();
    resp.text().await.unwrap()
}
