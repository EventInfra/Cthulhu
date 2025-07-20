use cthulhu_common::status::{JobCommand, JobUpdate};
use cthulhu_config::angel::AngelHeavenConfig;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::Sender;
use tokio_util::io::InspectReader;
use tracing::{debug, error, info, warn};

#[derive(Clone, Debug)]
pub struct MQTTSender {
    id: String,
    client: Option<AsyncClient>,
}

impl MQTTSender {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn empty() -> Self {
        Self {
            id: "".to_string(),
            client: None,
        }
    }

    pub fn with_client(client: AsyncClient, id: String) -> Self {
        Self {
            client: Some(client),
            id,
        }
    }
    pub async fn send_log_data(&self, data: &[u8]) -> color_eyre::Result<()> {
        if let Some(client) = &self.client {
            client
                .publish(format!("cthulhu/{}/serial", self.id), QoS::AtMostOnce, false, data)
                .await?;
        }
        Ok(())
    }

    pub async fn send_update(&self, update: JobUpdate) -> color_eyre::Result<()> {
        let data = serde_json::to_string(&update)?;
        if let Some(client) = &self.client {
            debug!("Sending update: {update:?}");
            client
                .publish(format!("cthulhu/{}/update", self.id), QoS::AtMostOnce, false, data)
                .await?;
        } else {
            debug!("Ignoring update.");
        }
        Ok(())
    }
}

pub async fn wrap_mqtt_serial_log<IO: 'static + AsyncRead + AsyncWrite + Unpin + Send + Sync>(
    inp: IO,
    mqtt_sender: MQTTSender,
) -> color_eyre::Result<impl 'static + AsyncRead + AsyncWrite + Unpin + Send + Sync> {
    let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
    tokio::spawn(async move {
        while let Some(msg) = receiver.recv().await {
            let r = mqtt_sender.send_log_data(&msg).await;
            if let Err(e) = r {
                error!("Error logging: {e}")
            }
        }
    });
    Ok(InspectReader::new(inp, move |d| {
        let _ = sender.send(d.to_vec());
    }))
}

async fn mqtt_options_from_config(config: &AngelHeavenConfig) -> color_eyre::Result<MqttOptions> {
    let mut mqttoptions = MqttOptions::new(&config.id, &config.host, config.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    Ok(mqttoptions)
}

pub async fn create_mqtt_sender_from_config(
    hconfig: &AngelHeavenConfig,
    tx: Sender<JobCommand>,
) -> color_eyre::Result<MQTTSender> {
    let (mqtt_client, mut mqtt_eventloop) =
        rumqttc::AsyncClient::new(mqtt_options_from_config(&hconfig).await?, 10);

    mqtt_client
        .subscribe(format!("cthulhu/{}/command", hconfig.id), QoS::AtLeastOnce)
        .await?;
    mqtt_client
        .subscribe(format!("cthulhu/command"), QoS::AtLeastOnce)
        .await?;

    let id = hconfig.id.clone();
    tokio::spawn(async move {
        loop {
            let r = mqtt_eventloop.poll().await;
            if let Ok(notification) = r {
                match notification {
                    Event::Incoming(Incoming::Publish(payload)) => {
                        if payload.topic == format!("cthulhu/{}/command", id) || payload.topic == "cthulhu/command" {
                            let command: JobCommand =
                                serde_json::from_slice(&payload.payload).unwrap();
                            info!("Received command: {command:?}");
                            if let Err(e) = tx.send(command).await {
                                warn!("Unable to TX command: {e:?}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    });

    Ok(MQTTSender::with_client(mqtt_client, hconfig.id.clone()))
}
