use crate::logging::TracingTarget;
use crate::mqtt::MQTTSender;
use chrono::Utc;
use color_eyre::eyre::Context;
use cthulhu_angel_sm::AngelJob;
use cthulhu_angel_sm::data_structure::{State, StateMachineTransition, StateMachineTrigger};
use cthulhu_angel_sm::state::StateMachine;
use cthulhu_common::devinfo::DeviceInformation;
use cthulhu_common::job::JobData;
use cthulhu_common::status::JobUpdate;
use std::collections::BTreeMap;
use std::path::PathBuf;
use swexpect::SwitchExpect;
use swexpect::hay::ReadUntil;
use tracing::{debug, info, warn};

pub struct ActiveJob {
    pub data: JobData,
    state_machine: StateMachine,
    shutdown_requested: bool,
    current_state: State,
    pub mqtt: MQTTSender,
    tracing_target: TracingTarget,
    rawlog_target: TracingTarget,
    log_dir: Option<PathBuf>,
    job_config: BTreeMap<String, String>,
}

impl AngelJob for ActiveJob {
    async fn init_job(&mut self) -> color_eyre::Result<()> {
        if let Some(log_dir) = self.log_dir.as_ref() {
            {
                let mut tracing_log_file = log_dir.clone();
                tracing_log_file.push(format!(
                    "{}--{}.log",
                    self.data.job_started.unwrap_or(Utc::now()).format("%Y-%m-%d--%H:%M:%S"),
                    self.mqtt.id()
                ));
                self.tracing_target.open_file(tracing_log_file)?;
            }
            {
                let mut raw_log_file = log_dir.clone();
                raw_log_file.push(format!(
                    "{}--{}.raw.log",
                    self.data.job_started.unwrap_or(Utc::now()).format("%Y-%m-%d--%H:%M:%S"),
                    self.mqtt.id()
                ));
                self.rawlog_target.open_file(raw_log_file)?;
            }
        }
        info!("Job initialized!");
        Ok(())
    }

    async fn finish_job(&mut self) -> color_eyre::Result<()> {
        info!("Job finished!");
        info!("Information items:");
        for i in self.data.info_items.iter() {
            info!(" - {i:?}");
        }
        self.send_update(JobUpdate::JobEnd(Utc::now())).await?;
        Ok(())
    }

    async fn reset(&mut self) -> color_eyre::Result<()> {
        info!("Resetting job...");
        //TODO: Maybe send a JobEnd sometimes?

        self.current_state = "Init".to_string();
        self.data.reset();
        self.send_update(JobUpdate::JobStart(Utc::now())).await?;
        self.send_update(JobUpdate::JobStageTransition(
            Utc::now(),
            self.current_state.clone(),
        ))
        .await?;
        Ok(())
    }

    async fn add_information(&mut self, information: DeviceInformation) -> color_eyre::Result<()> {
        info!("Recorded new switch information: {information:?}");
        self.send_update(JobUpdate::JobNewInfoItem(information))
            .await?;
        Ok(())
    }

    async fn get_job_config_key(&self, key: &str) -> Option<String> {
        self.job_config.get(key).cloned()
    }
}

impl ActiveJob {
    pub fn create(
        mqtt: MQTTSender,
        log_dir: Option<PathBuf>,
        tracing_target: TracingTarget,
        rawlog_target: TracingTarget,
        state_machine: StateMachine,
        job_config: BTreeMap<String, String>,
    ) -> Self {
        Self {
            data: JobData::with_label(mqtt.id()),
            current_state: "Init".to_string(),
            mqtt,
            log_dir,
            tracing_target,
            rawlog_target,
            state_machine,
            job_config,
            shutdown_requested: false,
        }
    }

    async fn transition(
        &mut self,
        t: &StateMachineTransition,
        p: &mut SwitchExpect,
        d: &str,
        m: &str,
    ) -> color_eyre::Result<()> {
        // Validate that the state exists
        let _ = self.state_machine.state(&t.target)?;

        let old_state = self.current_state.clone();
        self.current_state = t.target.clone();
        info!("State transition: {:?} -> {:?}", old_state, t.target);
        self.send_update(JobUpdate::JobStageTransition(Utc::now(), t.target.clone()))
            .await?;
        for action in &t.actions {
            action.perform(self, p, d, m).await?;
        }

        let cycles = self.data.state_history.iter().map(|(_, a)| a.as_str()).filter(|&s| s == self.current_state.as_str()).count();
        if cycles > 5 {
            warn!("Loop detected! Ending job...");
            self.add_information(DeviceInformation::LoopDetected).await?;
            self.current_state = "EndJob".to_string();
            self.send_update(JobUpdate::JobStageTransition(Utc::now(), self.current_state.clone()))
                .await?;
        }
        Ok(())
    }

    pub async fn step(&mut self, p: &mut SwitchExpect) -> color_eyre::Result<()> {
        let s = self.state_machine.state(&self.current_state)?;
        let transitions = &s.transitions;

        if let Some(t) = transitions
            .iter()
            .find(|t| t.trigger == StateMachineTrigger::Immediate)
        {
            self.transition(t, p, "", "")
                .await
                .context("process immediate transition")?;
        } else {
            let u = ReadUntil::Any(
                transitions
                    .iter()
                    .map(|t| t.trigger.to_needle().map(|v| v.unwrap()))
                    .collect::<color_eyre::Result<Vec<_>>>()?,
            );

            // Try to handle a result from the switches.
            debug!("Waiting for needle {u:?}...");
            let (d, m) = p
                .expect(&u)
                .await
                .context("failed to read from serial port")?;
            't_test: for t in transitions.iter() {
                if t.trigger.matches_result(&m)? {
                    self.transition(&t, p, &d, &m)
                        .await
                        .context("process serial transition")?;
                    break 't_test;
                }
            }
        }

        Ok(())
    }

    pub async fn flag_restart(&mut self) -> color_eyre::Result<()> {
        if self.data.get_status().is_idle() {
            panic!("Crash requested!");
        }
        self.shutdown_requested = true;
        Ok(())
    }

    async fn send_update(&mut self, update: JobUpdate) -> color_eyre::Result<()> {
        self.data.update(update.clone());
        self.mqtt.send_update(update).await?;
        if self.shutdown_requested && self.data.get_status().is_idle() {
            panic!("Crash requested!");
        }
        Ok(())
    }
}
