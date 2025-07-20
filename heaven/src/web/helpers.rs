use chrono::{DateTime, Utc};
use chrono_humanize::HumanTime;
use cthulhu_common::devinfo::DeviceInformation;
use cthulhu_common::job::{JobData, JobStatus};

pub trait PortStatusExt {
    fn get_css_backgroundcolor(&self) -> String;
}
impl PortStatusExt for JobStatus {
    fn get_css_backgroundcolor(&self) -> String {
        match self {
            JobStatus::Idle => "var(--primary-background)".to_string(),
            JobStatus::FinishSuccess => "#00ff00".to_string(),
            JobStatus::FinishWarning => "#ff9933".to_string(),
            JobStatus::FinishError => "#ff0000".to_string(),
            JobStatus::Busy => "#33bbff".to_string(),
            JobStatus::RunningLong => "#bb33ff".to_string(),
            JobStatus::Fatal => "#ff33dd".to_string(),
        }
    }
}


pub trait DateTimeAgo {
    fn timeago(&self) -> String;
}

impl DateTimeAgo for DateTime<Utc> {
    fn timeago(&self) -> String {
        let v = HumanTime::from(self.clone());
        let d = format!("{}", v);
        d
    }
}

pub fn get_dev_manuf(port: &JobData) -> String {
    for i in port.info_items.iter() {
        match i {
            DeviceInformation::Vendor(v) => return v.clone(),
            _ => {}
        }
    }
    "UNKN".to_string()
}

pub fn get_dev_model(port: &JobData) -> String {
    for i in port.info_items.iter() {
        match i {
            DeviceInformation::Model(v) => return v.clone(),
            _ => {}
        }
    }
    "UNKN".to_string()
}

pub fn get_dev_sn(port: &JobData) -> String {
    for i in port.info_items.iter() {
        match i {
            DeviceInformation::SerialNumber(v) => return v.clone(),
            _ => {}
        }
    }
    "UNKN".to_string()
}