use crate::args::Cli;
use crate::client::{NetboxClient, NetboxJournalEntryKind};
use clap::Parser;
use cthulhu_common::devinfo::{DeviceInformation, DeviceInformationType};
use cthulhu_common::job::JobData;
use cthulhu_common::status::{JobCommand, JobUpdate};
use cthulhu_config::netbox::{NetboxConfig, NetboxHeavenConfig, NetboxNBConfig};
use regex::Regex;
use rumqttc::{Event, Incoming, MqttOptions, QoS};
use std::collections::BTreeMap;
use std::time::Duration;
use tracing::{info, warn};
use cthulhu_config::LoadableConfig;

mod args;
mod client;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let args: Cli = Cli::parse();

    // install global subscriber configured based on RUST_LOG envvar.
    tracing_subscriber::fmt::init();

    let config = NetboxConfig::from_file(&args.config).await?;

    let nb_client = NetboxClient::with_url_and_token(&config.netbox.url, &config.netbox.token)?;

    info!("Connecting to MQTT...");
    let (mqtt_client, mut mqtt_eventloop) =
        rumqttc::AsyncClient::new(mqtt_options_from_config(&config.heaven).await?, 10);

    mqtt_client.subscribe("cthulhu/+/update", QoS::AtLeastOnce).await?;

    let update_re = Regex::new(r"cthulhu/(?<port_label>[^/]+)/update")?;
    let mut port_map: BTreeMap<String, JobData> = BTreeMap::new();

    info!("Sending GetJobData...");
    let cmd = JobCommand::GetJobData;
    let v = serde_json::to_string(&cmd)?;
    mqtt_client.publish("cthulhu/command".to_string(), QoS::AtLeastOnce, false, v).await?;

    info!("Running...");
    loop {
        let notification = mqtt_eventloop.poll().await?;
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                if let Some(caps) = update_re.captures(&publish.topic) {
                    let label = (&caps["port_label"]).to_string();
                    let update: JobUpdate = serde_json::from_slice(&publish.payload)?;
                    info!("Received update for {}.", label);

                    let data = port_map
                        .entry(label.clone())
                        .or_insert_with(|| JobData::with_label(&label));
                    data.update(update.clone());

                    if let JobUpdate::JobEnd(_) = update
                        && let Some(sn) = get_sn_from_job(data)
                    {
                        info!(
                            "Device with serial number {} on port {} has finished!",
                            sn, label
                        );
                        if let Err(e) = update_device(&nb_client, &config.netbox, &sn, data).await {
                            warn!("Unable to update device with ID {}: {}", sn, e);
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

async fn update_device(
    nb_client: &NetboxClient,
    nb_config: &NetboxNBConfig,
    sn: &str,
    data: &JobData,
) -> color_eyre::Result<()> {
    let device_id = nb_client.get_device_id_by_serial(&sn).await?;

    if data.get_max_information_type() != DeviceInformationType::Error {
        nb_client
            .set_device_status(device_id, &nb_config.target_status)
            .await?;
    }

    let mut comment = String::new();
    comment.push_str("# Cthulhu report\n");
    comment.push_str("\n\n");
    comment.push_str("## Job Information\n");
    comment.push_str("| Key | Value |\n");
    comment.push_str("| --- | ----- |\n");
    comment.push_str(&format!("| Max level | {:?} |\n", data.get_max_information_type()));
    comment.push_str(&format!("| Serial Port | {} |\n", data.label));
    if let Some(job_started) = data.job_started.as_ref() {
        comment.push_str(&format!("| Start Time | {} |\n", job_started));
    }
    if let Some(job_ended) = data.job_ended.as_ref() {
        comment.push_str(&format!("| End Time | {} |\n", job_ended));
    }
    if let Some(job_started) = data.job_started.as_ref() {
        comment.push_str(&format!(
            "| Log file | {}--{}.log |\n",
            job_started.format("%Y-%m-%d--%H:%M:%S"),
            data.label,
        ));
        comment.push_str(&format!(
            "| Raw log file | {}--{}.raw.log |\n",
            job_started.format("%Y-%m-%d--%H:%M:%S"),
            data.label,
        ));
    }
    comment.push_str("\n\n");
    comment.push_str("## Device Information Items\n");
    comment.push_str("| Level | Item |\n");
    comment.push_str("| ----- | ---- |\n");
    for item in data.info_items.iter() {
        comment.push_str(&format!("| {:?} | {:?} |\n", item.get_type(), item));
    }
    comment.push_str("\n\n");
    comment.push_str("## State history\n");
    comment.push_str("| Time | State |\n");
    comment.push_str("| ---- | ----- |\n");
    for (time, state) in data.state_history.iter() {
        comment.push_str(&format!("| {} | {} |\n", time, state));
    }

    let kind = match data.get_max_information_type() {
        DeviceInformationType::Info => NetboxJournalEntryKind::Success,
        DeviceInformationType::Warning => NetboxJournalEntryKind::Warning,
        DeviceInformationType::Error => NetboxJournalEntryKind::Danger,
    };

    nb_client.add_device_journal_entry(device_id, kind, &comment).await?;
    Ok(())
}

fn get_sn_from_job(data: &JobData) -> Option<String> {
    for item in data.info_items.iter() {
        if let DeviceInformation::SerialNumber(s) = item {
            return Some(s.clone());
        }
    }
    None
}

async fn mqtt_options_from_config(config: &NetboxHeavenConfig) -> color_eyre::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(&config.id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    Ok(mqttoptions)
}
