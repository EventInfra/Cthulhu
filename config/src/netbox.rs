use crate::LoadableConfig;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxConfig {
    #[serde(rename = "NetBox")]
    pub netbox: NetboxNBConfig,

    #[serde(rename = "Heaven")]
    pub heaven: NetboxHeavenConfig,
}
impl LoadableConfig for NetboxConfig {}

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxHeavenConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct NetboxNBConfig {
    pub token: String,
    pub url: String,
    pub target_status: String,
}
