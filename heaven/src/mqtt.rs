use cthulhu_common::status::{JobCommand, JobUpdate};
use regex::Regex;
use rumqttc::{AsyncClient, Event, EventLoop, Incoming, QoS};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Sender;
use tracing::trace;

#[derive(Clone, Debug)]
pub enum MQTTBroadcast {
    JobUpdate { label: String, update: JobUpdate },
    SerialData { label: String, data: Vec<u8> },
}

pub type BroadcastSender = Sender<MQTTBroadcast>;

pub fn create_broadcast() -> BroadcastSender {
    broadcast::channel(4096).0
}

pub async fn mqtt_main(
    sender: BroadcastSender,
    mqtt_client: AsyncClient,
    mut eventloop: EventLoop,
) -> color_eyre::Result<()> {
    mqtt_client.subscribe("+/update", QoS::AtLeastOnce).await?;
    mqtt_client.subscribe("+/serial", QoS::AtLeastOnce).await?;

    let update_re = Regex::new(r"(?<port_label>[^/]+)/update")?;
    let serial_re = Regex::new(r"(?<port_label>[^/]+)/serial")?;
    loop {
        let r = eventloop.poll().await?;
        match r {
            Event::Incoming(Incoming::Publish(publish)) => {
                if let Some(caps) = update_re.captures(&publish.topic) {
                    let label = (&caps["port_label"]).to_string();
                    let update: JobUpdate = serde_json::from_slice(&publish.payload)?;
                    let _ = sender.send(MQTTBroadcast::JobUpdate { label, update });
                }
                if let Some(caps) = serial_re.captures(&publish.topic) {
                    let label = (&caps["port_label"]).to_string();
                    let data = publish.payload.to_vec();
                    let _ = sender.send(MQTTBroadcast::SerialData { label, data });
                }
            }
            _ => {
                trace!("Ignoring unknown event.");
            }
        }
    }
}

#[derive(Clone)]
pub struct MQTTSender {
    client: AsyncClient,
}

impl MQTTSender {
    pub fn new(mqtt_client: AsyncClient) -> color_eyre::Result<Self> {
        Ok(Self {
            client: mqtt_client,
        })
    }

    pub async fn send_command(&self, port: &str, command: JobCommand) -> color_eyre::Result<()> {
        let data = serde_json::to_vec(&command)?;
        self.client
            .publish(format!("{}/command", port), QoS::AtMostOnce, false, data)
            .await?;
        Ok(())
    }
}
