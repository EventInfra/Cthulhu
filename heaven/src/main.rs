use crate::args::Cli;
use crate::manager::JobManager;
use crate::mqtt::MQTTSender;
use clap::Parser;
use cthulhu_config::heaven::{HeavenConfig, HeavenMQTTConfig};
use rumqttc::MqttOptions;
use std::str::FromStr;
use std::time::Duration;
use tokio::select;
use tokio::task::JoinHandle;
use tracing::level_filters::LevelFilter;
use tracing::{Level, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Layer, Registry};
use cthulhu_config::LoadableConfig;

mod args;
mod manager;
mod mqtt;
mod web;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    let config = HeavenConfig::from_file(&cli.config).await?;

    // Initialize logging
    let max_log_level =
        Level::from_str(&(config.log_level.as_ref().unwrap_or(&"info".to_string())))?;
    let stdsub =
        tracing_subscriber::fmt::layer().with_filter(LevelFilter::from_level(max_log_level));
    let subscriber = Registry::default().with(stdsub);
    tracing::subscriber::set_global_default(subscriber)?;

    info!("{config:?}");

    let (mqtt_client, mqtt_eventloop) =
        rumqttc::AsyncClient::new(mqtt_options_from_config(&config.mqtt).await?, 10);
    let mqtt_sender = MQTTSender::new(mqtt_client.clone())?;
    let mqtt_broadcast = mqtt::create_broadcast();

    let manager = JobManager::new().await?;

    let a = yeller(
        "web".to_string(),
        web::web_main(
            config.clone(),
            manager.clone(),
            mqtt_sender.clone(),
            mqtt_broadcast.clone(),
        ),
    )
    .await;
    let b = yeller(
        "mqtt".to_string(),
        mqtt::mqtt_main(mqtt_broadcast.clone(), mqtt_client, mqtt_eventloop),
    )
    .await;
    let c = yeller(
        "manager".to_string(),
        manager::manager_main(mqtt_broadcast, mqtt_sender, manager),
    )
    .await;

    select! {
        a = a => {
            tokio::time::sleep(Duration::from_secs(1)).await;
            a??;
        }
        b = b => {
            tokio::time::sleep(Duration::from_secs(1)).await;
            b??;
        }
        c = c => {
            tokio::time::sleep(Duration::from_secs(1)).await;
            c??;
        }
    }

    Ok(())
}

async fn yeller<T: Send + 'static, F: Future<Output = color_eyre::Result<T>> + Send + 'static>(
    label: String,
    f: F,
) -> JoinHandle<color_eyre::Result<T>> {
    tokio::spawn(async move {
        match f.await {
            Ok(v) => {
                warn!("Core task {label} has exited!");
                Ok(v)
            }
            Err(e) => {
                error!("Core task {label} has errored! {e}");
                Err(e)
            }
        }
    })
}

async fn mqtt_options_from_config(config: &HeavenMQTTConfig) -> color_eyre::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(
        config.id.as_ref().unwrap_or(&"heaven".to_string()),
        &config.host,
        config.port,
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    Ok(mqttoptions)
}
