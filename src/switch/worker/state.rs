use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tracing::info;
use rexpect::ReadUntil;
use crate::switch::worker::process::ProcessStage;
use crate::switch::worker::RexpectSession;
use crate::switch::worker::action::Action;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DeviceInformation {
    SerialNumber(String),
    SoftwareVersion(String),
    Model(String),
    Vendor(String),
    AttemptedToFixFilesystemIssues,
    ReadonlyFlash,
    SCSIErrors,
    KeptHostname,
}

impl DeviceInformation {
    pub fn get_type(&self) -> DeviceInformationType {
        match self {
            DeviceInformation::SerialNumber(_) => DeviceInformationType::Info,
            DeviceInformation::SoftwareVersion(_) => DeviceInformationType::Info,
            DeviceInformation::Model(_) => DeviceInformationType::Info,
            DeviceInformation::Vendor(_) => DeviceInformationType::Info,
            DeviceInformation::AttemptedToFixFilesystemIssues => DeviceInformationType::Warning,
            DeviceInformation::ReadonlyFlash => DeviceInformationType::Error,
            DeviceInformation::SCSIErrors => DeviceInformationType::Error,
            DeviceInformation::KeptHostname => DeviceInformationType::Warning,
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DeviceInformationType {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone)]
pub struct SwitchData {
    information: Vec<DeviceInformation>,
    job_started: DateTime<Utc>,
}

impl Default for SwitchData {
    fn default() -> Self {
        Self {
            information: Vec::new(),
            job_started: Utc::now(),
        }
    }
}

impl SwitchData {
    pub fn add_information(&mut self, information: DeviceInformation) {
        info!("Recorded new switch information: {information:?}");
        self.information.push(information);
    }

    pub fn get_information(&self) -> &[DeviceInformation] {
        &self.information
    }

    pub fn get_max_type(&self) -> Option<DeviceInformationType> {
        self.information.iter().map(|i| i.get_type()).max()
    }

    pub fn get_started(&self) -> DateTime<Utc> {
        self.job_started.clone()
    }
}

pub type ActionFunction = Box<dyn Fn(&mut SwitchData, &mut RexpectSession, &str, &str) -> color_eyre::Result<()>>;

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum StateCondition {
    WaitForString(String),
    WaitForRegex(String),
    Immediate,
}

impl StateCondition {
    pub fn to_needle(&self) -> color_eyre::Result<Option<ReadUntil>> {
        match self {
            StateCondition::WaitForString(s) => Ok(Some(ReadUntil::String(s.clone()))),
            StateCondition::WaitForRegex(s) => Ok(Some(ReadUntil::Regex(Regex::new(s)?))),
            StateCondition::Immediate => Ok(None),
        }
    }

    pub fn matches_result(&self, m: &str) -> color_eyre::Result<bool> {
        match self {
            StateCondition::WaitForString(s) => Ok(m == s),
            StateCondition::WaitForRegex(s) => {
                let r = Regex::new(s)?;
                Ok(r.is_match(m))
            }
            StateCondition::Immediate => Ok(true),
        }
    }
}



pub struct StateTransition {
    pub target_state: ProcessStage,
    pub actions: Vec<Action>,
    pub condition: StateCondition,
}
