use crate::AngelJob;
use crate::pfunc::ProcessFunction;
use crate::util::{vec_or_single, deser_duration};
use cthulhu_common::devinfo::DeviceInformation;
use serde::Deserialize;
use std::time::Duration;
use swexpect::SwitchExpect;
use tracing::warn;

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
#[serde(untagged)]
pub enum DeviceInfoArg {
    WithArgument(DeviceInformation),
    WithoutArgument { flag: DeviceInformation },
}

impl From<DeviceInfoArg> for DeviceInformation {
    fn from(value: DeviceInfoArg) -> Self {
        match value {
            DeviceInfoArg::WithArgument(e) => e,
            DeviceInfoArg::WithoutArgument { flag: e } => e,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
#[serde(tag = "type")]
pub enum Action {
    Send {
        text: String,
    },
    Flush,
    SendLine {
        line: String,
    },
    SendControl {
        char: char,
    },
    Function {
        func: ProcessFunction,
    },
    Repeat {
        #[serde(deserialize_with = "vec_or_single", rename = "action")]
        actions: Vec<Action>,
        times: usize,
    },
    Delay {
        #[serde(deserialize_with = "deser_duration")]
        duration: Duration,
    },
    AddDeviceInfo(DeviceInfoArg),
    FinishJob,
    SetupJob,
    SendConfigValue {
        key: String,
    },
}

impl Action {
    pub async fn perform<T: AngelJob>(
        &self,
        job: &mut T,
        p: &mut SwitchExpect,
        data: &str,
        mat: &str,
    ) -> color_eyre::Result<()> {
        match self {
            Action::Send { text: s } => {
                p.send(s).await?;
                Ok(())
            }
            Action::Flush => {
                p.flush().await?;
                Ok(())
            }
            Action::SendLine { line: s } => {
                p.send_line(s).await?;
                Ok(())
            }
            Action::SendControl { char: c } => {
                p.send_control(*c).await?;
                Ok(())
            }
            Action::Function { func: pf } => pf.execute(job, p, data, mat).await,
            Action::FinishJob => {
                job.finish_job().await?;
                Ok(())
            }
            Action::Repeat {
                actions: a,
                times: t,
            } => {
                for _ in 0..*t {
                    for b in a.iter() {
                        Box::pin(b.perform(job, p, data, mat)).await?;
                    }
                }
                Ok(())
            }
            Action::Delay { duration: d } => {
                tokio::time::sleep(*d).await;
                Ok(())
            }
            Action::AddDeviceInfo(i) => {
                job.add_information(i.clone().into()).await?;
                Ok(())
            }
            Action::SetupJob => {
                job.init_job().await?;
                Ok(())
            }
            Action::SendConfigValue { key } => {
                if let Some(v) = job.get_job_config_key(key).await {
                    p.send(&v).await?;
                } else {
                    warn!("No such config item: {key}");
                }
                Ok(())
            }
        }
    }
}
