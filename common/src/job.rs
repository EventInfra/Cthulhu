use crate::devinfo::{DeviceInformation, DeviceInformationType};
use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Display;
use std::ops::Add;
use crate::status::JobUpdate;

/// Current and historical data of a job.
#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct JobData {
    /// Port label
    pub label: String,
    /// When did the job start?
    pub job_started: Option<DateTime<Utc>>,
    pub job_ended: Option<DateTime<Utc>>,
    /// History of states, 0 == oldest, n == newest
    pub state_history: Vec<(DateTime<Utc>, String)>,
    /// List of device information
    pub info_items: HashSet<DeviceInformation>,
}

impl JobData {
    pub fn with_label(label: &str) -> Self {
        JobData {
            label: label.to_string(),
            job_started: None,
            job_ended: None,
            state_history: Vec::new(),
            info_items: HashSet::new(),
        }
    }

    pub fn reset(&mut self) {
        self.job_started = None;
        self.job_ended = None;
        self.state_history = Vec::new();
        self.info_items = HashSet::new();
    }

    pub fn update(&mut self, update: JobUpdate) {
        match update {
            JobUpdate::JobStageTransition(d, s) => {
                self.state_history.push((d, s));
            }
            JobUpdate::JobStart(d) => {
                self.reset();
                self.job_started = Some(d);
            }
            JobUpdate::JobEnd(d) => {
                self.job_ended = Some(d);
            }
            JobUpdate::JobNewInfoItem(i) => {
                self.info_items.insert(i);
            }
            JobUpdate::JobFullData(d) => {
                *self = d;
            }
        }
    }

    pub fn get_current_stage(&self) -> Option<&str> {
        self.state_history.last().map(|(_, s)| s.as_str())
    }

    pub fn get_last_updated(&self) -> Option<DateTime<Utc>> {
        self.state_history.last().map(|(s, _)| s.clone())
    }

    pub fn get_max_information_type(&self) -> DeviceInformationType {
        self.info_items.iter().map(|i| i.get_type())
            .max()
            .unwrap_or(DeviceInformationType::Warning)
    }
    pub fn get_status(&self) -> JobStatus {
        if let Some(current_state) = self.get_current_stage() {
            match current_state {
                "Init" => JobStatus::Idle,
                "SwitchDetect" => JobStatus::Idle,
                "JobFinished" => {
                    match self.get_max_information_type() {
                        DeviceInformationType::Info => JobStatus::FinishSuccess,
                        DeviceInformationType::Warning => JobStatus::FinishWarning,
                        DeviceInformationType::Error => JobStatus::FinishError,
                    }
                }
                _ => {
                    if self
                        .get_last_updated()
                        .unwrap_or(Utc::now())
                        .add(TimeDelta::new(60 * 15, 0).unwrap())
                        < Utc::now()
                    {
                        JobStatus::RunningLong
                    } else {
                        JobStatus::Busy
                    }
                }
            }
        } else {
            JobStatus::Idle
        }
    }
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum JobStatus {
    /// Initial state
    #[default]
    Idle,
    /// Waiting for new switch; previous success
    FinishSuccess,
    /// Waiting for new switch; previous warning
    FinishWarning,
    /// Waiting for new switch; previous error
    FinishError,
    /// Working on a switch.
    Busy,
    /// This job is taking too long.
    RunningLong,
    /// This thread has crashed.
    Fatal,
}

impl JobStatus {
    pub fn is_idle(&self) -> bool {
        match self {
            JobStatus::Idle => true,
            JobStatus::FinishSuccess => true,
            JobStatus::FinishWarning => true,
            JobStatus::FinishError => true,
            _ => false,
        }
    }

    pub fn is_finished(&self) -> bool {
        match self {
            JobStatus::FinishSuccess => true,
            JobStatus::FinishWarning => true,
            JobStatus::FinishError => true,
            _ => false,
        }
    }
}

impl Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JobStatus::Idle => write!(f, "😴"),
            JobStatus::FinishSuccess => write!(f, "✅"),
            JobStatus::FinishWarning => write!(f, "⚠️"),
            JobStatus::FinishError => write!(f, "❗"),
            JobStatus::Busy => write!(f, "⏳"),
            JobStatus::RunningLong => write!(f, "⏰"),
            JobStatus::Fatal => write!(f, "😵"),
        }
    }
}
