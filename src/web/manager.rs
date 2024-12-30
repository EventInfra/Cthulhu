use std::fmt::Display;
use std::future::Future;
use std::ops::Add;
use std::sync::Arc;
use chrono::{DateTime, TimeDelta, Utc};
use futures::future::join_all;
use futures::FutureExt;
use serde::Serialize;
use std::fs::{File, OpenOptions};
use std::io::Write;
use tokio::sync::RwLock;
use crate::switch::{DeviceInformation, PortStatus, PortUpdate, ProcessStage, SpawnedPort};

#[derive(Default, Debug, Serialize, Clone)]
pub struct PortManagerEntry {
    pub label: String,
    pub last_update: DateTime<Utc>,
    pub job_started: DateTime<Utc>,
    pub current_stage: ProcessStage,
    pub status: PortStatus,
    pub info_items: Vec<DeviceInformation>,
}

impl PortStatus {
    pub fn get_css_backgroundcolor(&self) -> String {
        match self {
            PortStatus::Idle => "#ffffff".to_string(),
            PortStatus::IdleSuccess => "#00ff00".to_string(),
            PortStatus::IdleWarning => "#ff9933".to_string(),
            PortStatus::IdleError => "#ff0000".to_string(),
            PortStatus::Busy => "#33bbff".to_string(),
            PortStatus::RunningLong => "#bb33ff".to_string(),
            PortStatus::Fatal => "#ff33dd".to_string(),
        }
    }
}

impl Display for PortStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortStatus::Idle => write!(f, "😴"),
            PortStatus::IdleSuccess => write!(f, "✅"),
            PortStatus::IdleWarning => write!(f, "⚠️"),
            PortStatus::IdleError => write!(f, "❗"),
            PortStatus::Busy => write!(f, "⏳"),
            PortStatus::RunningLong => write!(f, "⏰"),
            PortStatus::Fatal => write!(f, "😵"),
        }
    }
}

struct PortManagerInner {
    ports: Vec<PortManagerEntry>,
}

#[derive(Clone)]
pub struct PortManager {
    inner: Arc<RwLock<PortManagerInner>>,
    manager_log: Arc<RwLock<File>>,
}

impl PortManager {
    pub async fn new() -> color_eyre::Result<Self> {
        let f = OpenOptions::new().create(true).append(true).open("logs/manager.log")?;
        Ok(Self {
            inner: Arc::new(RwLock::new( PortManagerInner {
                ports: Vec::new(),
            })),
            manager_log: Arc::new(RwLock::new(f)),
        })
    }

    pub async fn get_ports(&self) -> Vec<PortManagerEntry> {
        let r = self.inner.read().await;
        r.ports.iter().cloned().map(|mut p| {
            if p.last_update.add(TimeDelta::new(60*15, 0).unwrap()) < Utc::now() && !p.status.is_idle() {
                p.status = PortStatus::RunningLong;
            }
            p
        }).collect()
    }

    pub async fn spawn(ports: Vec<SpawnedPort>) -> color_eyre::Result<(Self, impl Future<Output = ()>)> {
        let manager = Self::new().await?;
        let manager_copy = manager.clone();
        let jobs = ports.into_iter()
            .map(|p|
                tokio::spawn({
                    let manager = manager.clone();
                    async move {
                    port_update_listener(p, manager).await
                }})
            );
        Ok((manager_copy, join_all(jobs).map(|_| ())))
    }

    pub async fn accept_update(&self, port_label: &str, update_ts: DateTime<Utc>, update: PortUpdate) -> color_eyre::Result<()> {
        let mut inner = self.inner.write().await;
        let existing = if let Some(existing) = inner.ports.iter_mut().find(|p| p.label == port_label) {
            existing
        } else {
            inner.ports.push(PortManagerEntry {
                label: port_label.to_string(),
                ..Default::default()
            });
            inner.ports.last_mut().unwrap()
        };

        let t = Utc::now().to_rfc3339();
        let mut log = self.manager_log.write().await;

        existing.last_update = update_ts;
        match update {
            PortUpdate::PortStateTransition(old, new) => {
                existing.current_stage = new;
                writeln!(log, "{t}\t{port_label}\tSTATE\t{old:?}->{new:?}")?;
            }
            PortUpdate::PortStatusUpdate(new) => {
                let old = existing.status;
                writeln!(log, "{t}\t{port_label}\tSTATUS\t{old:?}->{new:?}")?;
                existing.status = new;
            }
            PortUpdate::PortJobStart(s) => {
                writeln!(log, "{t}\t{port_label}\tJOB_START")?;
                existing.job_started = s;
                existing.info_items.clear();
            }
            PortUpdate::PortNewInfoItem(i) => {
                writeln!(log, "{t}\t{port_label}\tINFO\t{i:?}")?;
                existing.info_items.push(i);
            }
        }
        Ok(())
    }
}

#[allow(irrefutable_let_patterns)]
async fn port_update_listener(port: SpawnedPort, manager: PortManager) -> color_eyre::Result<()> {
    while let (ts, update) = port.receiver.recv().await? {
        manager.accept_update(&port.config.label, ts, update).await?;
    }
    Ok(())
}
