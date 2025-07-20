use crate::mqtt::{BroadcastSender, MQTTBroadcast, MQTTSender};
use cthulhu_common::status::{JobCommand, JobUpdate};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::broadcast::error::RecvError;
use tracing::warn;
use cthulhu_common::job::JobData;

#[derive(Default, Debug, Serialize, Clone)]
pub struct PortManagerEntry {
    pub data: JobData,
    pub log_buffer: Vec<u8>,
}

struct JobManagerInner {
    ports: Vec<PortManagerEntry>,
}

impl JobManagerInner {
    fn get_port_mut(&mut self, port_label: &str) -> &mut PortManagerEntry {
        let existing_index = self
            .ports
            .iter()
            .enumerate()
            .find(|(_, x)| x.data.label == port_label)
            .map(|(i, _)| i);
        if let Some(index) = existing_index {
            self.ports.get_mut(index).unwrap()
        } else {
            self.ports.push(PortManagerEntry {
                data: JobData::with_label(port_label),
                ..Default::default()
            });
            self.ports.last_mut().unwrap()
        }
    }
}

#[derive(Clone)]
pub struct JobManager {
    inner: Arc<RwLock<JobManagerInner>>,
}

impl JobManager {
    pub async fn new() -> color_eyre::Result<Self> {
        Ok(Self {
            inner: Arc::new(RwLock::new(JobManagerInner { ports: Vec::new() })),
        })
    }

    pub async fn get_ports(&self) -> Vec<PortManagerEntry> {
        let r = self.inner.read().await;
        r.ports
            .iter()
            .cloned()
            .collect()
    }

    pub async fn get_port(&self, label: &str) -> Option<PortManagerEntry> {
        let r = self.inner.read().await;
        r.ports
            .iter()
            .find(|p| p.data.label == label)
            .cloned()
    }

    async fn append_log_data(&self, port_label: &str, data: &[u8]) -> color_eyre::Result<()> {
        let mut inner = self.inner.write().await;
        let existing = inner.get_port_mut(port_label);
        existing.log_buffer.extend(data);
        Ok(())
    }
    async fn accept_update(&self, port_label: &str, update: JobUpdate) -> color_eyre::Result<()> {
        let mut inner = self.inner.write().await;
        let existing = inner.get_port_mut(port_label);

        match &update {
            JobUpdate::JobStart(_) => {
                existing.log_buffer = Vec::new();
            }
            _ => {}
        }

        existing.data.update(update);

        Ok(())
    }
}

pub async fn manager_main(
    broadcast: BroadcastSender,
    sender: MQTTSender,
    manager: JobManager,
) -> color_eyre::Result<()> {
    let mut receiver = broadcast.subscribe();

    sender.broadcast_command(JobCommand::GetJobData).await?;
    
    loop {
        let msg = receiver.recv().await;
        match msg {
            Ok(MQTTBroadcast::JobUpdate { label, update }) => {
                manager.accept_update(&label, update).await?;
            }
            Ok(MQTTBroadcast::SerialData { label, data }) => {
                manager.append_log_data(&label, &data).await?;
            }
            Err(RecvError::Lagged(n)) => {
                warn!("Skipping {n} messages!");
            }
            Err(e) => {
                return Err(e.into());
            }
        }
    }
}
