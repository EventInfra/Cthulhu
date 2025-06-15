use crate::mqtt::{BroadcastSender, MQTTBroadcast};
use chrono::{DateTime, TimeDelta, Utc};
use cthulhu_common::devinfo::DeviceInformation;
use cthulhu_common::stages::ProcessStage;
use cthulhu_common::status::{JobUpdate, PortJobStatus};
use serde::Serialize;
use std::ops::Add;
use std::sync::Arc;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::RwLock;
use tracing::warn;

#[derive(Default, Debug, Serialize, Clone)]
pub struct PortManagerEntry {
    pub label: String,
    pub last_update: DateTime<Utc>,
    pub job_started: DateTime<Utc>,
    pub current_stage: ProcessStage,
    pub status: PortJobStatus,
    pub info_items: Vec<DeviceInformation>,
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
            .find(|(_, x)| x.label == port_label)
            .map(|(i, _)| i);
        if let Some(index) = existing_index {
            self.ports.get_mut(index).unwrap()
        } else {
            self.ports.push(PortManagerEntry {
                label: port_label.to_string(),
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
            .map(|mut p| {
                if p.last_update.add(TimeDelta::new(60 * 15, 0).unwrap()) < Utc::now()
                    && !p.status.is_idle()
                {
                    p.status = PortJobStatus::RunningLong;
                }
                p
            })
            .collect()
    }

    pub async fn get_port(&self, label: &str) -> Option<PortManagerEntry> {
        let r = self.inner.read().await;
        r.ports
            .iter()
            .find(|p| p.label == label)
            .cloned()
            .map(|mut p| {
                if p.last_update.add(TimeDelta::new(60 * 15, 0).unwrap()) < Utc::now()
                    && !p.status.is_idle()
                {
                    p.status = PortJobStatus::RunningLong;
                }
                p
            })
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

        existing.last_update = Utc::now();

        match update {
            JobUpdate::JobStageTransition(_old, new) => {
                existing.current_stage = new;
            }
            JobUpdate::JobStatusUpdate(new) => {
                existing.status = new;
            }
            JobUpdate::JobStart(s) => {
                existing.job_started = s;
                existing.info_items.clear();
                existing.log_buffer.clear();
            }
            JobUpdate::JobNewInfoItem(i) => {
                existing.info_items.push(i);
            }
        }
        Ok(())
    }
}

pub async fn manager_main(
    broadcast: BroadcastSender,
    manager: JobManager,
) -> color_eyre::Result<()> {
    let mut receiver = broadcast.subscribe();
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
