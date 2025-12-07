use std::path::Path;
use serde::de::DeserializeOwned;
use tracing::info;

pub mod angel;
pub mod heaven;
pub mod octhulhu;
pub mod netbox;
pub mod provision;

#[allow(async_fn_in_trait)]
pub trait LoadableConfig where Self: DeserializeOwned + Sized {
    async fn from_file<P: AsRef<Path>>(p: P) -> color_eyre::Result<Self> {
        info!("Using config file: {}", p.as_ref().display());
        let d = tokio::fs::read_to_string(p).await?;
        let d = toml::from_str(d.as_str())?;
        Ok(d)
    }
}
