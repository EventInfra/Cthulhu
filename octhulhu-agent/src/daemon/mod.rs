use std::collections::BTreeMap;
use std::time::Duration;
use regex::Regex;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, MqttOptions, QoS};
use serial2_tokio::SerialPort;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::info;
use cthulhu_common::status::{JobCommand, JobUpdate};
use cthulhu_config::octhulhu::{OcthulhuConfig, OcthulhuHeavenConfig};
use crate::daemon::tracker::PortTracker;
use crate::serial;

mod tracker;

async fn mqtt_options_from_config(config: &OcthulhuHeavenConfig) -> color_eyre::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(&config.id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    Ok(mqttoptions)
}

pub async fn daemon(conf: OcthulhuConfig) -> color_eyre::Result<()> {
    info!("Opening serial ports...");
    let mut ports: BTreeMap<String, SerialPort> = BTreeMap::new();
    for k in conf.port_mapping.keys() {
        ports.insert(k.clone(), serial::connect_serial_by_sn(&k).await?);
    }
    info!("Opened {} ports!", ports.len());

    info!("Connecting to MQTT...");
    let (mqtt_client, mqtt_eventloop) =
        rumqttc::AsyncClient::new(mqtt_options_from_config(&conf.heaven).await?, 10);

    info!("Starting MQTT thread...");
    let mut handles = Vec::new();
    let port_tracker = PortTracker::new();

    {
        let port_tracker = port_tracker.clone();
        handles.push(tokio::task::spawn(async move {
            mqtt_handler(port_tracker, mqtt_eventloop).await.unwrap();
        }));
    }


    info!("Setting up port tracker...");
    for (id, serial_port) in ports.iter() {
        for (port_idx, label) in conf.port_mapping[id].iter().enumerate() {
            port_tracker.add_port(label, serial_port.try_clone()?, port_idx as u8, &id).await;
            mqtt_client
                .subscribe(format!("cthulhu/{}/update", label), QoS::AtLeastOnce)
                .await?;
            let cmd = JobCommand::GetJobData;
            let v = serde_json::to_string(&cmd)?;
            mqtt_client.publish(format!("cthulhu/{}/command", label), QoS::AtLeastOnce, false, v).await?;

        }
    }


    info!("Starting tasks...");
    for (id, port) in ports.into_iter() {
        let tracker = port_tracker.clone();
        let mqtt = mqtt_client.clone();
        handles.push(tokio::task::spawn(async move {
            serial_handler(&id, tracker, port, mqtt).await.unwrap();
        }));
    }

    info!("Running!");
    for h in handles {
        h.await?;
    }

    Ok(())
}

async fn mqtt_handler(port_tracker: PortTracker, mut eventloop: EventLoop) -> color_eyre::Result<()> {
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

async fn serial_handler(sn: &str, tracker: PortTracker, port: SerialPort, mqtt: AsyncClient) -> color_eyre::Result<()> {
    let d = "M\r\nP\r\n";
    port.write_all(d.as_bytes()).await?;

    let mut br = BufReader::new(port);
    loop {
        let mut l = String::new();
        br.read_line(&mut l).await?;
        let mut cs = l.trim().chars();
        let c = cs.next().unwrap();

        match c {
            'P' => {
                for i in 0..8u8 {
                    let c = cs.next().unwrap();
                    let v = c == '1';
                    tracker.serial_switch_presence_update(sn, i, v, mqtt.clone()).await?;
                }
            }
            'M' => {
                for i in 0..8u8 {
                    let c = cs.next().unwrap();
                    let v = c == '1';
                    tracker.serial_module_presence_update(sn, i, v, mqtt.clone()).await?;
                }
            }
            _ => {}
        }
    }
}