use chrono::{DateTime, Utc};
use cthulhu_common::devinfo::DeviceInformation;
use cthulhu_common::status::PortJobStatus;
use include_dir::{Dir, include_dir};
use std::collections::HashMap;
use std::sync::LazyLock;
use tera::{Tera, Value};
use tokio::sync::RwLock;

trait PortStatusExt {
    fn get_css_backgroundcolor(&self) -> String;
}
impl PortStatusExt for PortJobStatus {
    fn get_css_backgroundcolor(&self) -> String {
        match self {
            PortJobStatus::Idle => "var(--primary-background)".to_string(),
            PortJobStatus::FinishSuccess => "#00ff00".to_string(),
            PortJobStatus::FinishWarning => "#ff9933".to_string(),
            PortJobStatus::FinishError => "#ff0000".to_string(),
            PortJobStatus::Busy => "#33bbff".to_string(),
            PortJobStatus::RunningLong => "#bb33ff".to_string(),
            PortJobStatus::Fatal => "#ff33dd".to_string(),
        }
    }
}

fn csscolor_filter(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let status: PortJobStatus = serde_json::from_value(input.clone())?;
    let color = status.get_css_backgroundcolor();
    Ok(serde_json::to_value(color)?)
}

fn render_devinfo(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let status: DeviceInformation = serde_json::from_value(input.clone())?;
    let color = format!("{status:?}");
    Ok(serde_json::to_value(color)?)
}

fn format_status(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: PortJobStatus = serde_json::from_value(input.clone())?;
    let d = format!("{}", v);
    Ok(serde_json::to_value(d)?)
}

fn is_finished(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: PortJobStatus = serde_json::from_value(input.clone())?;
    let d = v.is_finished();
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
            _ => {}
        }
    }
    Ok(serde_json::to_value("UNKN")?)
}

fn get_sn(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: Vec<DeviceInformation> = serde_json::from_value(input.clone())?;
    for i in v {
        match i {
            DeviceInformation::SerialNumber(v) => return Ok(serde_json::to_value(v)?),
            _ => {}
        }
    }
    Ok(serde_json::to_value("UNKN")?)
}

fn get_model(input: &Value, _args: &HashMap<String, Value>) -> tera::Result<Value> {
    let v: Vec<DeviceInformation> = serde_json::from_value(input.clone())?;
    for i in v {
        match i {
            DeviceInformation::Model(v) => return Ok(serde_json::to_value(v)?),
            _ => {}
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
    let mut t = Tera::default();
    for f in TEMPLATE_DIR.files() {
        t.add_raw_template(f.path().to_str().unwrap(), f.contents_utf8().unwrap())
            .expect("Failed to add template");
    }
    t.register_filter("csscolor", csscolor_filter);
    t.register_filter("format_status", format_status);
    t.register_filter("is_finished", is_finished);
    t.register_filter("format_timeago", format_timeago);
    t.register_filter("column_calc", column_calc);
    t.register_filter("get_manuf", get_manuf);
    t.register_filter("get_model", get_model);
    t.register_filter("get_sn", get_sn);
    t.register_filter("render_devinfo", render_devinfo);
    t
}

static TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/web/templates");

pub static TERA: LazyLock<RwLock<Tera>> = LazyLock::new(|| RwLock::new(make_tera()));
