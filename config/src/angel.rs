use std::collections::BTreeMap;
use serde::Deserialize;
use std::path::PathBuf;
use crate::LoadableConfig;

#[derive(Deserialize, Debug, Clone)]
pub struct AngelConfig {
    pub log_level: Option<String>,
    pub log_dir: Option<PathBuf>,
    #[serde(default = "default_active_states")]
    pub active_states: Vec<String>,

    #[serde(rename = "JobConfig", default)]
    pub job_config: BTreeMap<String, String>,

    #[serde(flatten)]
    pub port: AngelPortConfig,
    #[serde(rename = "Heaven")]
    pub heaven: Option<AngelHeavenConfig>,
}

fn default_active_states() -> Vec<String> {
    vec!["wipe".to_string()]
}

impl LoadableConfig for AngelConfig {}

#[derive(Deserialize, Debug, Clone)]
pub struct AngelHeavenConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub enum AngelPortConfig {
    TTY(TTYConfig),
    RawTCP(RawTCPConfig),
}

#[derive(Deserialize, Debug, Clone)]
pub struct TTYConfig {
    pub path: PathBuf,
    #[serde(default)]
    pub baudrate: TTYBaudrate,
}

#[derive(Deserialize, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub struct TTYBaudrate(pub u32);

impl Default for TTYBaudrate {
    fn default() -> Self {
        Self(9600)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RawTCPConfig {
    pub endpoint: String,
}
