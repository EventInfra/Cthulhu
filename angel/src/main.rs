use crate::args::Cli;
use crate::job::ActiveJob;
use crate::logging::{SerialLogger, setup_tracing, wrap_raw_serial_log};
use crate::mqtt::{MQTTSender, create_mqtt_sender_from_config, wrap_mqtt_serial_log};
use crate::ports::port_from_config;
use clap::Parser;
use color_eyre::eyre::eyre;
use cthulhu_angel_sm::AngelJob;
use cthulhu_angel_sm::builder::StateMachineBuilder;
use cthulhu_common::status::{JobCommand, JobUpdate};
use cthulhu_config::angel::AngelConfig;
use swexpect::SwitchExpect;
use tokio::sync::mpsc;
use tracing::info;
use cthulhu_config::LoadableConfig;

mod args;
mod job;
mod logging;
mod mqtt;
mod ports;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let cli = Cli::parse();
    let config = AngelConfig::from_file(&cli.config).await?;
    let tracing_target = setup_tracing(&config).await?;

    info!("{config:?}");

    let (tx, mut rx) = mpsc::channel(100);
    let mqtt_sender = if let Some(hconfig) = config.heaven.as_ref() {
        create_mqtt_sender_from_config(hconfig, tx).await?
    } else {
        MQTTSender::empty()
    };

    let port = port_from_config(&config.port).await?;
    let port = SerialLogger::new(port);
    let port = wrap_mqtt_serial_log(port, mqtt_sender.clone()).await?;
    let (port, rawlog_target) = wrap_raw_serial_log(port).await?;
    let mut p = SwitchExpect::new(port, None);

    let mut smb = StateMachineBuilder::new();
    smb.load_builtin_state_files()?;
    for id in config.active_states.iter() {
        smb.activate_state_file(id)?;
    }
    let sm = smb.build()?;

    let mut job = ActiveJob::create(
        mqtt_sender.clone(),
        config.log_dir.clone(),
        tracing_target,
        rawlog_target,
        sm,
        config.job_config.clone(),
    );
    job.reset().await?;

    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(cmd) = msg {
                    match cmd {
                        JobCommand::ResetJob => {
                            job.reset().await?;
                        },
                        JobCommand::RestartAngel => {
                            job.flag_restart().await?;
                        },
                        JobCommand::GetJobData => {
                            mqtt_sender.send_update(JobUpdate::JobFullData(job.data.clone())).await?;
                        },
                    }
                } else {
                    return Err(eyre!("MQTT broken."));
                }
            },
            r = job.step(&mut p) => {
                r?;
            },
        }
    }
}
