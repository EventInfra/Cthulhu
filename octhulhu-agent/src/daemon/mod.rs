use crate::daemon::tracker::PortTracker;
use crate::serial;
use crate::serial::discovery::DiscoveredDevice;
use crate::serial::{SerialPortManager, SerialPortMessage};
use cthulhu_common::status::{JobCommand, JobUpdate};
use cthulhu_config::octhulhu::{OcthulhuConfig, OcthulhuHeavenConfig};
use regex::Regex;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use std::time::Duration;
use tracing::{info, warn};

mod tracker;

async fn mqtt_options_from_config(
    config: &OcthulhuHeavenConfig,
) -> color_eyre::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(&config.id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    Ok(mqttoptions)
}

pub async fn daemon(conf: OcthulhuConfig) -> color_eyre::Result<()> {
    let discovered_boards = serial::discovery::discover_devices(&conf).await?;
    let boards: Vec<DiscoveredDevice> = discovered_boards
        .into_iter()
        .filter(|p| conf.port_mapping.contains_key(&p.serial_number))
        .collect();
    info!("Opening serial ports...");
    let serial_port_manager = serial::SerialPortManager::new();
    for board in boards.iter() {
        board.connect_to(&serial_port_manager).await?;
    }
    info!("Opened {} ports!", boards.len());

    info!("Connecting to MQTT...");
    let (mqtt_client, mqtt_eventloop) =
        AsyncClient::new(mqtt_options_from_config(&conf.heaven).await?, 10);

    info!("Starting MQTT thread...");
    let mut handles = Vec::new();
    let port_tracker = PortTracker::with_serial_port_manager(serial_port_manager.clone());

    {
        let port_tracker = port_tracker.clone();
        handles.push(tokio::task::spawn(async move {
            mqtt_handler(port_tracker, mqtt_eventloop).await.unwrap();
        }));
    }

    info!("Setting up port tracker...");
    for dev in boards.iter() {
        let id = dev.serial_number.as_str();
        for (port_idx, label) in conf.port_mapping[id].iter().enumerate() {
            port_tracker.add_port(label, port_idx as u8, &id).await;
            mqtt_client
                .subscribe(format!("cthulhu/{}/update", label), QoS::AtLeastOnce)
                .await?;
            let cmd = JobCommand::GetJobData;
            let v = serde_json::to_string(&cmd)?;
            mqtt_client
                .publish(
                    format!("cthulhu/{}/command", label),
                    QoS::AtLeastOnce,
                    false,
                    v,
                )
                .await?;
        }
    }

    info!("Starting the serial handler...");
    handles.push(tokio::task::spawn(async move {
        serial_handler(serial_port_manager, port_tracker, mqtt_client)
            .await
            .unwrap();
    }));

    info!("Running!");
    for h in handles {
        h.await?;
    }

    Ok(())
}

async fn mqtt_handler(
    port_tracker: PortTracker,
    mut eventloop: EventLoop,
) -> color_eyre::Result<()> {
    let update_re = Regex::new(r"cthulhu/(?<port_label>[^/]+)/update")?;
    loop {
        let notification = eventloop.poll().await?;
        match notification {
            Event::Incoming(Incoming::Publish(publish)) => {
                if let Some(caps) = update_re.captures(&publish.topic) {
                    let label = (&caps["port_label"]).to_string();
                    let update: JobUpdate = serde_json::from_slice(&publish.payload)?;
                    info!("Received update for {}.", label);
                    port_tracker.mqtt_update(&label, update).await?;
                }
            }
            _ => {}
        }
    }
}

async fn serial_handler(
    serial_port_manager: SerialPortManager,
    tracker: PortTracker,
    mqtt: AsyncClient,
) -> color_eyre::Result<()> {
    let mut receive_channel = serial_port_manager.receiver();

    serial_port_manager.request_all_module_updates().await?;
    serial_port_manager.request_all_presence_updates().await?;

    loop {
        let message = receive_channel.recv().await?;
        match message {
            SerialPortMessage::UnknownResponse {
                serial_number,
                line,
            } => {
                warn!("Unknown board response received from {serial_number}: {line}");
            }
            SerialPortMessage::ModuleUpdate {
                serial_number,
                present,
            } => {
                for (i, v) in present.into_iter().enumerate() {
                    tracker
                        .serial_module_presence_update(&serial_number, i as u8, v, mqtt.clone())
                        .await?;
                }
            }
            SerialPortMessage::PresenceUpdate {
                serial_number,
                present,
            } => {
                for (i, v) in present.into_iter().enumerate() {
                    tracker
                        .serial_switch_presence_update(&serial_number, i as u8, v, mqtt.clone())
                        .await?;
                }
            }
        }
    }
}
