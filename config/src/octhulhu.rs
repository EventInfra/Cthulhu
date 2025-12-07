use crate::LoadableConfig;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Deserialize, Debug, Clone)]
pub struct OcthulhuConfig {
    #[serde(rename = "Heaven")]
    pub heaven: OcthulhuHeavenConfig,
    #[serde(rename = "NetworkSerial", default)]
    pub network_serials: Vec<OcthulhuNetworkSerial>,
    #[serde(rename = "PortMapping", default)]
    pub port_mapping: BTreeMap<String, Vec<String>>,
}

impl LoadableConfig for OcthulhuConfig {}

#[derive(Deserialize, Debug, Clone)]
pub struct OcthulhuHeavenConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct OcthulhuNetworkSerial {
    pub host: String,
    pub port: u16,
}
