use cthulhu_common::devinfo::DeviceInformation;

pub mod action;
pub mod builder;
pub mod data_structure;
pub mod pfunc;
pub mod state;
pub mod trigger;

mod util;

//TODO: Figure out how to properly fix the warning.
#[allow(async_fn_in_trait)]
pub trait AngelJob {
    async fn init_job(&mut self) -> color_eyre::Result<()>;
    async fn finish_job(&mut self) -> color_eyre::Result<()>;
    async fn reset(&mut self) -> color_eyre::Result<()>;
    async fn add_information(&mut self, information: DeviceInformation) -> color_eyre::Result<()>;
    async fn get_job_config_key(&self, key: &str) -> Option<String>;
}
