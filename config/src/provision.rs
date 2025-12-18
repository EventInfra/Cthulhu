use std::path::PathBuf;
use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Deserializer};
use crate::LoadableConfig;

#[derive(Deserialize, Debug, Clone)]
pub struct ProvisionConfig {
    pub log_level: Option<String>,

    pub config_server: String,

    #[serde(rename = "Web")]
    pub web: ProvisionWebConfig,

    #[serde(rename = "ModelOSMapping", default)]
    pub model_os_mappings: Vec<ProvisionModelOSMapping>,

    #[serde(rename = "AutoReload")]
    pub autoreload_config: ProvisionAutoReloadConfig,

    pub ntp_server: String,
}

impl LoadableConfig for ProvisionConfig {}

#[derive(Deserialize, Debug, Clone)]
pub struct ProvisionAutoReloadConfig {
    pub snafu_host: String,
    pub deploy_host: String,
    pub ping_target: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProvisionWebConfig {
    #[serde(default)]
    pub listen_address: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProvisionModelOSMapping {
    pub vendor: String,
    #[serde(deserialize_with = "deserialize_regex")]
    pub model: Regex,
    #[serde(deserialize_with = "deserialize_regex")]
    pub target_version: Regex,
    pub os_image: PathBuf,
}

fn deserialize_regex<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where D: Deserializer<'de> {
    let r = String::deserialize(deserializer)?;
    RegexBuilder::new(r.as_str()).case_insensitive(true).build().map_err(serde::de::Error::custom)
}
