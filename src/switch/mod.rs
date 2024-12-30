use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::config::PortConfig;
use crate::switch::worker::worker_function;

pub mod connecter;
mod worker;
pub use worker::process::ProcessStage;
pub use crate::switch::worker::state::DeviceInformation;

pub enum PortUpdate {
    PortStateTransition(ProcessStage, ProcessStage),
    PortStatusUpdate(PortStatus),
    PortJobStart(DateTime<Utc>),
    PortNewInfoItem(DeviceInformation),
}

pub type PortUpdateChannelType = (DateTime<Utc>, PortUpdate);
pub type PortUpdateReceiver = async_channel::Receiver<PortUpdateChannelType>;

#[derive(Clone)]
pub struct PortUpdateSender {
    inner: async_channel::Sender<PortUpdateChannelType>,
}

impl PortUpdateSender {
    pub fn send(&self, update: PortUpdate) -> color_eyre::Result<DateTime<Utc>> {
        let t = Utc::now();
        self.inner.send_blocking((t.clone(), update))?;
        Ok(t)
    }
}

pub fn run_port_sync(config: &PortConfig) -> color_eyre::Result<()> {
    let (update_sender, _update_receiver) = async_channel::unbounded::<PortUpdateChannelType>();
    let update_sender = PortUpdateSender { inner: update_sender };

    let (r, w) = config.connect()?;

    worker_function(config, update_sender, r, w)?;

    Ok(())
}

pub struct SpawnedPort {
    pub config: PortConfig,
    pub receiver: PortUpdateReceiver,
}

pub fn spawn_port(config: PortConfig) -> color_eyre::Result<SpawnedPort> {
    let (update_sender, update_receiver) = async_channel::unbounded::<PortUpdateChannelType>();
    let update_sender = PortUpdateSender { inner: update_sender };

    let (r, w) = config.connect()?;
    let config_copy = config.clone();

    std::thread::Builder::new().name(format!("port {}", config.path))
        .spawn(move || {
            let res = worker_function(&config, update_sender.clone(), r, w);
            if let Err(e) = res {
                eprintln!("Worker thread for port {} exited with error: {}", config.path, e);
                update_sender.send(PortUpdate::PortStatusUpdate(PortStatus::Fatal)).unwrap();
            } else {
                eprintln!("Worker thread for port {} exited without error!", config.path);
            }
            println!("AAAAAA");
        })?;

    Ok(SpawnedPort {
        config: config_copy,
        receiver: update_receiver,
    })
}

#[derive(Default, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum PortStatus {
    /// Initial state
    #[default]
    Idle,
    /// Waiting for new switch; previous success
    IdleSuccess,
    /// Waiting for new switch; previous warning
    IdleWarning,
    /// Waiting for new switch; previous error
    IdleError,
    /// Working on a switch.
    Busy,
    /// This job is taking too long.
    RunningLong,
    /// This thread has crashed.
    Fatal,
}

impl PortStatus {
    pub fn is_idle(&self) -> bool {
        match self {
            PortStatus::Idle => true,
            PortStatus::IdleSuccess => true,
            PortStatus::IdleWarning => true,
            PortStatus::IdleError => true,
            _ => false,
        }
    }
}
