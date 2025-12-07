use crate::state::AppStateHandle;
use askama::Template;
use axum::body::Body;
use axum::extract::{Path, Query, State};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::Host;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

pub fn get_script_routes() -> Router<AppStateHandle> {
    Router::new()
        .route("/provision.sh", get(get_stage1))
        .route("/stage2.sh", get(get_stage2))
        .route("/jinstall/{file}", get(get_jinstall))
        .route("/assets/{*path}", get(static_path))
}

#[derive(Template)]
#[template(path = "junos-stage1.sh")]
struct JunosStage1Template {
    base_url: String,
}

async fn get_stage1(Host(host): Host) -> impl IntoResponse {
    let data = JunosStage1Template {
        base_url: format!("http://{host}"),
    };

    (StatusCode::OK, data.render().unwrap())
}
#[derive(Template)]
#[template(path = "junos-stage2.sh")]
struct JunosStage2Template {
    base_url: String,
}

#[derive(Template)]
#[template(path = "junos-stage2-os-change.sh")]
struct JunosStage2UpgradeTemplate {
    base_url: String,
    target_jinstall: String,
}

#[derive(Deserialize, Debug)]
struct Stage2Query {
    junos: Option<String>,
    sku: Option<String>,
}

async fn get_stage2(State(state): State<AppStateHandle>, Host(host): Host, Query(query): Query<Stage2Query>) -> Response {
    if let Some(junos) = query.junos && let Some(sku) = query.sku {
        for os_mapping in state.os_mappings.iter() {
            if os_mapping.vendor == "Juniper" && os_mapping.model.is_match(&sku)  && !os_mapping.target_version.is_match(&junos) {
                // We need to upgrade the OS.
                let data = JunosStage2UpgradeTemplate {
                    base_url: format!("http://{host}"),
                    target_jinstall: os_mapping.os_image.file_name().unwrap().to_str().unwrap().to_string(),
                };

                return (StatusCode::OK, data.render().unwrap()).into_response();
            }
        }
    }

    let data = JunosStage2Template {
        base_url: format!("http://{host}"),
    };

    (StatusCode::OK, data.render().unwrap()).into_response()
}

async fn get_jinstall(State(state): State<AppStateHandle>, Path(path): Path<String>) -> Response {
    for os_mapping in state.os_mappings.iter() {
        if os_mapping.vendor == "Juniper" {
            if os_mapping.os_image.ends_with(&path) {
                let f = File::open(&os_mapping.os_image).await;
                let Ok(f) = f else { return (StatusCode::NOT_FOUND, "Unable to find jinstall").into_response(); };
                let reader = ReaderStream::new(f);
                let body = Body::from_stream(reader);
                return (StatusCode::OK, body).into_response();
            }
        }
    }

    (StatusCode::NOT_FOUND, "Unable to find jinstall").into_response()
}



static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/juniper/assets");


async fn static_path(State(state): State<AppStateHandle>, Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or_text_plain();

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap(),
        Some(file) => {
            let data = file.contents();
            let data = match String::from_utf8(data.to_vec()) {
                Ok(s) => {
                    s.replace("%%{snafu_host}%%", state.autoreload.snafu_host.as_str())
                        .replace("%%{deploy_host}%%", state.autoreload.deploy_host.as_str())
                        .replace("%%{ping_target}%%", state.autoreload.ping_target.as_str())
                        .into_bytes()
                }
                Err(_) => data.to_vec(),
            };
            Response::builder()
                .status(StatusCode::OK)
                .header(
                    header::CONTENT_TYPE,
                    HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                )
                .body(Body::from(data))
                .unwrap()
        }
    }
}