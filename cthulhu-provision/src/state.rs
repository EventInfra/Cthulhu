use std::sync::Arc;
use cthulhu_config::provision::{ProvisionAutoReloadConfig, ProvisionModelOSMapping};

pub struct AppState {
    pub config_server: String,
    pub os_mappings: Vec<ProvisionModelOSMapping>,
    pub autoreload: ProvisionAutoReloadConfig,
}

pub type AppStateHandle = Arc<AppState>;
