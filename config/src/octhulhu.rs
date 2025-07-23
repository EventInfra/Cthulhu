use std::collections::BTreeMap;
use std::path::Path;
use serde::Deserialize;
use tracing::info;

#[derive(Deserialize, Debug, Clone)]
pub struct OcthulhuConfig {
    #[serde(rename = "Heaven")]
    pub heaven: OcthulhuHeavenConfig,
    #[serde(rename = "PortMapping", default)]
    pub port_mapping: BTreeMap<String, Vec<String>>,
}
impl OcthulhuConfig {
    pub async fn from_file<P: AsRef<Path>>(p: P) -> color_eyre::Result<Self> {
        info!("Using config file: {}", p.as_ref().display());
        let d = tokio::fs::read_to_string(p).await?;
        let d = toml::from_str(d.as_str())?;
        Ok(d)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct OcthulhuHeavenConfig {
    pub id: String,
    pub host: String,
    pub port: u16,
}
