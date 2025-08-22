use cthulhu_common::job::{JobData, JobStatus};
use cthulhu_common::status::{JobCommand, JobUpdate};
use rumqttc::{AsyncClient, QoS};
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};
use crate::serial::SerialPortManager;

#[derive(Clone)]
pub struct PortTracker {
    serial_port_manager: SerialPortManager,
    inner: Arc<Mutex<PortTrackerInner>>,
}

impl PortTracker {
    pub fn with_serial_port_manager(serial_port_manager: SerialPortManager) -> Self {
        Self {
            inner: Arc::new(Mutex::new(PortTrackerInner {
                entries: BTreeMap::new(),
            })),
            serial_port_manager,
        }
    }

    pub async fn add_port(
        &self,
        label: &str,
        port_idx: u8,
        board_sn: &str,
    ) {
        let mut inner = self.inner.lock().await;
        inner.entries.insert(
            label.to_string(),
            PortTrackerEntry {
                data: JobData::with_label(label),
                serial_port_manager: self.serial_port_manager.clone(),
                port_idx,
                board_sn: board_sn.to_string(),
                module_present: None,
                switch_present: None,
            },
        );
    }

    pub async fn mqtt_update(&self, label: &str, update: JobUpdate) -> color_eyre::Result<()> {
        let mut inner = self.inner.lock().await;
        if let Some(pentry) = inner.entries.get_mut(&label.to_string()) {
            pentry.mqtt_update(update).await?;
        }
        Ok(())
    }

    pub async fn serial_module_presence_update(
        &self,
        bsn: &str,
        port_idx: u8,
        v: bool,
        mqtt: AsyncClient,
    ) -> color_eyre::Result<()> {
        let mut inner = self.inner.lock().await;
        for entry in inner.entries.values_mut() {
            if entry.board_sn.as_str() == bsn && entry.port_idx == port_idx {
                entry.module_presence_update(v, mqtt).await?;
                break;
            }
        }
        Ok(())
    }

    pub async fn serial_switch_presence_update(
        &self,
        bsn: &str,
        port_idx: u8,
        v: bool,
        mqtt: AsyncClient,
    ) -> color_eyre::Result<()> {
        let mut inner = self.inner.lock().await;
        for entry in inner.entries.values_mut() {
            if entry.board_sn.as_str() == bsn && entry.port_idx == port_idx {
                entry.switch_presence_update(v, mqtt).await?;
                break;
            }
        }
        Ok(())
    }
}

struct PortTrackerInner {
    entries: BTreeMap<String, PortTrackerEntry>,
}

struct PortTrackerEntry {
    data: JobData,
    serial_port_manager: SerialPortManager,
    board_sn: String,
    port_idx: u8,
    module_present: Option<bool>,
    switch_present: Option<bool>,
}

impl PortTrackerEntry {
    async fn update_led_color(&mut self) -> color_eyre::Result<()> {
        let (r, g, b) = match self.data.get_status() {
            JobStatus::Idle => (127, 127, 127),
            JobStatus::FinishSuccess => (0, 255, 0),
            JobStatus::FinishWarning => (0xff, 0x99, 0x33),
            JobStatus::FinishError => (255, 0, 0),
            JobStatus::Busy => (0x0, 0x0, 0xff),
            JobStatus::RunningLong => (0xbb, 0x33, 0xff),
            JobStatus::Fatal => (0xff, 0x33, 0xdd),
        };
        debug!("Color: {} {} {}", r, g, b);
        if self.switch_present.unwrap_or(false) && self.module_present.unwrap_or(false) {
            self.serial_port_manager.set_led_color(
                &self.board_sn,
                self.port_idx,
                r,
                g,
                b
            ).await?;
        } else {
            self.serial_port_manager.set_led_color(&self.board_sn, self.port_idx, 0xc7, 0x15, 0x85).await?;
        }
        Ok(())
    }
    async fn mqtt_update(&mut self, update: JobUpdate) -> color_eyre::Result<()> {
        self.data.update(update);
        self.update_led_color().await?;
        Ok(())
    }

    async fn module_presence_update(
        &mut self,
        v: bool,
        _mqtt: AsyncClient,
    ) -> color_eyre::Result<()> {
        let old = self.module_present;
        self.module_present = Some(v);
        debug!("Module presence update for {}: {}", self.data.label, v);
        if !old.unwrap_or(false) && self.module_present.unwrap_or(false) {
            info!("Module {} is plugged in!", self.data.label);
        }
        if old.unwrap_or(false) && !self.module_present.unwrap_or(false) {
            info!("Module {} is unplugged!", self.data.label);
        }

        self.update_led_color().await?;
        Ok(())
    }

    async fn switch_presence_update(
        &mut self,
        v: bool,
        mqtt: AsyncClient,
    ) -> color_eyre::Result<()> {
        let old = self.switch_present;
        self.switch_present = Some(v);
        debug!("Switch presence update for {}: {}", self.data.label, v);
        self.update_led_color().await?;
        if !old.unwrap_or(false) && self.switch_present.unwrap_or(false) {
            info!("Switch {} is plugged in!", self.data.label);
        }
        if old.unwrap_or(false) && !self.switch_present.unwrap_or(false) {
            info!("Switch {} is unplugged!", self.data.label);
        }
        if old.is_some() && self.module_present.unwrap_or(false) {
            if v && !old.unwrap()  {
                info!("Resetting job for {}...", self.data.label);
                let cmd = JobCommand::ResetJob;
                let v = serde_json::to_string(&cmd)?;
                mqtt.publish(
                    format!("cthulhu/{}/command", self.data.label),
                    QoS::AtLeastOnce,
                    false,
                    v,
                )
                .await?;
            }
        }
        Ok(())
    }
}
