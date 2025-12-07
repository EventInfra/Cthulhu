use serde::Deserialize;
use crate::LoadableConfig;

#[derive(Deserialize, Debug, Clone)]
pub struct HeavenConfig {
    pub log_level: Option<String>,

    #[serde(rename = "Web")]
    pub web: HeavenWebConfig,
    #[serde(rename = "MQTT")]
    pub mqtt: HeavenMQTTConfig,
}

impl LoadableConfig for HeavenConfig {}

#[derive(Deserialize, Debug, Clone)]
pub struct HeavenWebConfig {
    pub listen_address: String,
}
#[derive(Deserialize, Debug, Clone)]
pub struct HeavenMQTTConfig {
    pub id: Option<String>,
    pub host: String,
    pub port: u16,
}
