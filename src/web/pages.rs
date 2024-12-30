use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use axum::extract::{Path, State};
use axum::response::{Html, IntoResponse};
use chrono::{DateTime, Utc};
use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use tera::{Tera, Value};
use tera_template_macro::TeraTemplate;
use crate::switch::{DeviceInformation, PortStatus};
use crate::web::manager::{PortManager, PortManagerEntry};

fn csscolor_filter(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let status: PortStatus = serde_json::from_value(input.clone())?;
    let color = status.get_css_backgroundcolor();
    Ok(serde_json::to_value(color)?)
}

fn format_status(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: PortStatus = serde_json::from_value(input.clone())?;
    let d = format!("{}", v);
    Ok(serde_json::to_value(d)?)
}

fn format_timeago(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: DateTime<Utc> = serde_json::from_value(input.clone())?;
    let v = chrono_humanize::HumanTime::from(v);
    let d = format!("{}", v);
    Ok(serde_json::to_value(d)?)
}

fn get_manuf(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: Vec<DeviceInformation> = serde_json::from_value(input.clone())?;
    for i in v {
        match i {
            DeviceInformation::Vendor(v) => return Ok(serde_json::to_value(v)?),
            _ => {},
        }
    }
    Ok(serde_json::to_value("UNKN")?)
}

fn get_sn(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: Vec<DeviceInformation> = serde_json::from_value(input.clone())?;
    for i in v {
        match i {
            DeviceInformation::SerialNumber(v) => return Ok(serde_json::to_value(v)?),
            _ => {},
        }
    }
    Ok(serde_json::to_value("UNKN")?)
}

fn get_model(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: Vec<DeviceInformation> = serde_json::from_value(input.clone())?;
    for i in v {
        match i {
            DeviceInformation::Model(v) => return Ok(serde_json::to_value(v)?),
            _ => {},
        }
    }
    Ok(serde_json::to_value("UNKN")?)
}


fn column_calc(input: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
    // This could be done within tera itself; but time crunch
    let v: usize = serde_json::from_value(input.clone())?;
    let d: usize = serde_json::from_value(args.get("height").unwrap().clone())?;
    let t = ((v as f64) / d as f64).ceil() as usize;
    Ok(serde_json::to_value(t)?)
}

fn make_tera() -> Tera {
    // let mut t = Tera::new("src/web/templates/**/*").expect("Failed to create Tera instance");
    let mut t = Tera::default();
    for f in TEMPLATE_DIR.files() {
        t.add_raw_template(f.path().to_str().unwrap(), f.contents_utf8().unwrap()).expect("Failed to add template");
    }
    t.register_filter("csscolor", csscolor_filter);
    t.register_filter("format_status", format_status);
    t.register_filter("format_timeago", format_timeago);
    t.register_filter("column_calc", column_calc);
    t.register_filter("get_manuf", get_manuf);
    t.register_filter("get_model", get_model);
    t.register_filter("get_sn", get_sn);
    t
}

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/web/templates");


pub static TERA: LazyLock<RwLock<Tera>> = LazyLock::new(|| {
    RwLock::new(make_tera())
});
#[derive(TeraTemplate, Serialize)]
#[template(path="index.html")]
pub struct IndexPageTemplate {
    ports: Vec<PortManagerEntry>,
}
#[derive(TeraTemplate, Serialize)]
#[template(path="port.html")]
pub struct PortPageTemplate {
    port: PortManagerEntry,
    log: String,
}
pub async fn index(State(state): State<PortManager>) -> impl IntoResponse {
    let mut context = IndexPageTemplate {
        ports: state.get_ports().await,
    };

    context.ports.sort_by(|a, b| a.label.cmp(&b.label));

    Html(context.render(TERA.read().unwrap().clone()))
}

pub async fn logs_page(State(state): State<PortManager>, Path(port_label): Path<String>) -> impl IntoResponse {
    let ports = state.get_ports().await;
    let port = ports.into_iter().find(|p| p.label == port_label).unwrap();
    let b = PathBuf::new().join("logs/").join(format!("{}-{}.log", port.label, port.job_started.format("%Y-%m-%d-%H-%M-%S")));
    let context = PortPageTemplate {
        port,
        log: strip_ansi_escapes::strip_str(tokio::fs::read_to_string(&b).await.unwrap()),
    };

    Html(context.render(TERA.read().unwrap().clone()))
}
