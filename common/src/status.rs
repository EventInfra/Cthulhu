use crate::devinfo::DeviceInformation;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::job::JobData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobUpdate {
    JobStageTransition(DateTime<Utc>, String),
    JobStart(DateTime<Utc>),
    JobEnd(DateTime<Utc>),
    JobNewInfoItem(DeviceInformation),
    JobFullData(JobData),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JobCommand {
    ResetJob,
    RestartAngel,
    GetJobData,
}
