use crate::action::Action;
use crate::pfunc::ProcessFunction;
use crate::state::{StateCondition, StateTransition};
use cthulhu_common::devinfo::DeviceInformation;
use cthulhu_common::stages::ProcessStage;
use cthulhu_common::status::PortJobStatus;
use std::time::Duration;

pub trait ProcessStageTransition {
    fn get_transitions(&self) -> color_eyre::Result<Vec<StateTransition>>;
}
impl ProcessStageTransition for ProcessStage {
    fn get_transitions(&self) -> color_eyre::Result<Vec<StateTransition>> {
        match self {
            ProcessStage::Init => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::SwitchDetect,
                    condition: StateCondition::Immediate,
                    actions: vec![
                        Action::SetupJob,
                    ],
                },
            ]),
            ProcessStage::SwitchDetect => Ok(vec![
                //EX3300/EX2200
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 1.1".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //EX4400
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Booting from Flash A".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Primary BIOS version CDEN_P_EX1".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //EX2300C
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 2016".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //EX2300C
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Juniper U-Boot Script File".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //EX4100
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 2021".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //QFX10k
                StateTransition {
                    target_state: ProcessStage::Junos23WaitForBootloader,
                    condition: StateCondition::WaitForString("Juniper Linux".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                //Arista
                StateTransition {
                    target_state: ProcessStage::AristaWaitForBootloader,
                    condition: StateCondition::WaitForString("Aboot".to_string()),
                    actions: vec![
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                // This transition is to account for the console server in the bitlair rack.
                StateTransition {
                    target_state: ProcessStage::SwitchDetect,
                    condition: StateCondition::WaitForString("A non-empty Data Buffering File was found.".to_string()),
                    actions: vec![
                        Action::SendLine("E".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForBootloader => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader2,
                    condition: StateCondition::WaitForString("scanning bus for devices".to_string()),
                    actions: vec![
                        Action::SendControl('c'),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForBootloader2 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader3,
                    condition: StateCondition::WaitForString("=>".to_string()),
                    actions: vec![
                        Action::SendLine("setenv boot_unattended".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForBootloader3 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader4,
                    condition: StateCondition::WaitForString("=>".to_string()),
                    actions: vec![
                        Action::SendLine("boot".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForBootloader4 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosEnterSingleUserMode,
                    condition: StateCondition::WaitForString("space bar for command prompt".to_string()),
                    actions: vec![
                        Action::Repeat(vec![
                            Action::Send(" ".to_string()),
                            Action::Flush,
                        ], 10),
                    ]
                },
            ]),
            ProcessStage::JunosEnterSingleUserMode => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitBoot,
                    condition: StateCondition::WaitForString("loader>".to_string()),
                    actions: vec![
                        Action::Repeat(vec![
                            Action::SendControl('h'),
                            Action::Flush,
                        ], 10),
                        Action::SendLine("".to_string()),
                        Action::SendLine("boot -s".to_string()),
                    ]
                }
            ]),
            ProcessStage::JunosAwaitBoot => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader,
                    condition: StateCondition::WaitForString("U-Boot 1.1".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::BootLoop),
                        Action::UpdatePortStatus(PortJobStatus::Busy),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosAwaitRecoveryShell,
                    condition: StateCondition::WaitForString("Enter full pathname of shell or 'recovery' for root password recovery or RETURN for /bin/sh:".to_string()),
                    actions: vec![
                        Action::SendLine("recovery".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosAwaitBoot,
                    condition: StateCondition::WaitForString("SCSI Status".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::SCSIErrors),
                    ],
                },
            ]),
            ProcessStage::JunosAwaitRecoveryShell => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitRecoveryShell,
                    condition: StateCondition::WaitForString("SCSI Status".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::SCSIErrors),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("CLI invocation failed".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::FailedToEnterSingleUserMode),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosAnswerZeroize,
                    condition: StateCondition::WaitForRegex(r"\{[a-z0-9]+:0\}".to_string()),
                    actions: vec![
                        Action::Delay(Duration::from_secs(3)),
                        Action::SendLine("request system zeroize".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("continue, shell, abort, retry, or reboot".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::OSCorruption),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosWaitForBootloader,
                    condition: StateCondition::WaitForString("error: filesystem consistency checks (fsck -p -y) failed".to_string()),
                    actions: vec![
                        Action::SendLine("shell".to_string()),
                        Action::Function(ProcessFunction::FixFS),
                        Action::Delay(Duration::from_secs(3)),
                    ],
                },
            ]),
            ProcessStage::JunosAnswerZeroize => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitZeroizeFinish,
                    condition: StateCondition::WaitForString("Erase all data, including configuration and log files? [yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosAwaitZeroizeFinish => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosLogin,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("root".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosAwaitZeroizeFinish,
                    condition: StateCondition::WaitForString("Unable to load a kernel!".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::UnableToLoadAKernel),
                    ],
                },
            ]),
            ProcessStage::JunosLogin => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForString("root@:RE:0%".to_string()),
                    actions: vec![
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForRegex(r"root@[A-Za-z0-9\-]+:RE:0%".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::KeptHostname),
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForString("root@:LC:0%".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::StrangeCLIPrompt),
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosHappyCli,
                    condition: StateCondition::WaitForRegex(r"root@[A-Za-z0-9\-]+:LC:0%".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::KeptHostname),
                        Action::AddDeviceInfo(DeviceInformation::StrangeCLIPrompt),
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.* /var/log/* /var/tmp/*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli,
                    condition: StateCondition::WaitForString("Please re-install JUNOS".to_string()),
                    actions: vec![
                        Action::SendLine("echo \"y\" | crontab -r".to_string()),
                        Action::SendLine("rm -rfv /var/tmp/autoreload.* /tmp/autoreload.* /var/core/core.*  /var/log/* /var/tmp/*".to_string()),
                        Action::SendLine("cli".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosHappyCli => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosVersionOutput,
                    condition: StateCondition::WaitForRegex(r"root>".to_string()),
                    actions: vec![
                        Action::SendLine("show version | no-more".to_string())
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosVersionOutput,
                    condition: StateCondition::WaitForRegex(r"root@[A-Za-z0-9\-]+>".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::KeptHostname),
                        Action::SendLine("show version | no-more".to_string()),
                    ],
                },
            ]),
            ProcessStage::JunosBackupImageCli => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli2,
                    condition: StateCondition::WaitForRegex(r"root>".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::AlternateImage),
                        Action::SendLine("request system snapshot media internal slice alternate".to_string()),
                    ],
                },
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli2,
                    condition: StateCondition::WaitForRegex(r"root@[A-Za-z0-9\-]+>".to_string()),
                    actions: vec![
                        Action::AddDeviceInfo(DeviceInformation::KeptHostname),
                        Action::SendLine("request system snapshot media internal slice alternate".to_string())
                    ],
                },
            ]),
            ProcessStage::JunosBackupImageCli2 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli3,
                    condition: StateCondition::WaitForString("continue? [yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string())
                    ],
                },
            ]),

            ProcessStage::JunosBackupImageCli3 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosBackupImageCli4,
                    condition: StateCondition::WaitForRegex(r"root(@[A-Za-z0-9\-]+)?>".to_string()),
                    actions: vec![
                        Action::SendLine("request system reboot slice alternate media internal in 0".to_string())
                    ],
                },
            ]),
            ProcessStage::JunosBackupImageCli4 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosAwaitBackupImageCliRecoveryFinish,
                    condition: StateCondition::WaitForString("[yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string())
                    ],
                },
            ]),
            ProcessStage::JunosAwaitBackupImageCliRecoveryFinish => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosLogin,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("root".to_string()),
                    ],
                },
            ]),

            ProcessStage::JunosVersionOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosChassisOutput,
                    condition: StateCondition::WaitForRegex(r"root(@[A-Za-z0-9\-]+)?>".to_string()),
                    actions: vec![
                        Action::Function(ProcessFunction::CaptureJunosVersion),
                        Action::SendLine("show chassis hardware | no-more".to_string()),
                    ],
                }
            ]),

            ProcessStage::JunosChassisOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosPoweroffConfirm,
                    condition: StateCondition::WaitForRegex(r"root(@[A-Za-z0-9\-]+)?>".to_string()),
                    actions: vec![
                        Action::Function(ProcessFunction::CaptureChassisOutput),
                        Action::SendLine("request system power-off in 0".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosPoweroffConfirm => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosWaitForPoweroff,
                    condition: StateCondition::WaitForString("[yes,no] (no)".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                },
                // Some platforms only take halt, not power-off
                StateTransition {
                    target_state: ProcessStage::JunosPoweroffConfirm,
                    condition: StateCondition::WaitForString("error: command is not valid".to_string()),
                    actions: vec![
                        Action::SendLine("request system halt in 0".to_string()),
                    ],
                }
            ]),
            ProcessStage::JunosWaitForPoweroff => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("has halted.".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("acpi0: Powering system off".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("Rebooting...".to_string()),
                    actions: vec![],
                },
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("reboot: Power down".to_string()),
                    actions: vec![],
                },
            ]),

            ProcessStage::AristaWaitForBootloader => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaWipeStartupConfig,
                    condition: StateCondition::WaitForString("Press Control-C now to enter Aboot shell".to_string()),
                    actions: vec![
                        Action::SendControl('c'),
                    ],
                },
            ]),

            ProcessStage::AristaWipeStartupConfig => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaRebootAfterStartupConfigWipe,
                    condition: StateCondition::WaitForString("Aboot#".to_string()),
                    actions: vec![
                        Action::SendLine("rm -r /mnt/flash/.persist".to_string()),
                        Action::SendLine("rm /mnt/flash/startup-config".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaRebootAfterStartupConfigWipe => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaWaitForReboot,
                    condition: StateCondition::WaitForString("Aboot#".to_string()),
                    actions: vec![
                        Action::SendLine("exit".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaWaitForReboot => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaLoggingIn,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("admin".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaLoggingIn => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::AristaVersionOutput,
                    condition: StateCondition::WaitForString("localhost>".to_string()),
                    actions: vec![
                        Action::SendLine("show version".to_string()),
                        Action::SendLine("enable".to_string()),
                        Action::SendLine("bash rm -rfv /var/core/core.*".to_string()),
                        Action::SendLine("exit".to_string()),
                    ],
                }
            ]),

            ProcessStage::AristaVersionOutput => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::EndJob,
                    condition: StateCondition::WaitForString("localhost>".to_string()),
                    actions: vec![
                        Action::Function(ProcessFunction::CaptureAristaVersion),
                    ],
                }
            ]),

            ProcessStage::Junos23WaitForBootloader => Ok(vec![
                StateTransition {
                    condition: StateCondition::WaitForString("seconds... (press Ctrl-C to interrupt)".to_string()),
                    target_state: ProcessStage::Junos23Bootloader1,
                    actions: vec![
                        Action::SendControl('c'),
                    ],
                },
                // StateTransition {
                //     condition: StateCondition::WaitForString("localhost login:".to_string()),
                //     target_state: ProcessStage::Junos23Bootloader1,
                //     actions: vec![
                //         Action::SendControl('c'),
                //     ],
                // },
            ]),

            ProcessStage::Junos23Bootloader1 => Ok(vec![StateTransition {
                condition: StateCondition::WaitForString("Choice:".to_string()),
                target_state: ProcessStage::Junos23Bootloader2,
                actions: vec![
                    Action::Send("5".to_string()),
                    Action::Flush,
                ],
            }]),
            ProcessStage::Junos23Bootloader2 => Ok(vec![StateTransition {
                condition: StateCondition::WaitForString("Choice:".to_string()),
                target_state: ProcessStage::Junos23AwaitRecoveryShell,
                actions: vec![
                    Action::Send("2".to_string()),
                    Action::Flush,
                ],
            }]),
            ProcessStage::Junos23AwaitRecoveryShell => Ok(vec![
                StateTransition {
                    condition: StateCondition::WaitForRegex(r"\{[a-z0-9]+:0\}".to_string()),
                    target_state: ProcessStage::Junos23AnswerZeroize,
                    actions: vec![
                        Action::SendLine("request system zeroize".to_string()),
                    ],
                }
            ]),
            ProcessStage::Junos23AnswerZeroize => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::Junos23AwaitZeroizeFinish,
                    condition: StateCondition::WaitForString("Erase all data, including configuration and log files?. In case of Dual RE system, both Routing Engines will be zeroized".to_string()),
                    actions: vec![
                        Action::SendLine("yes".to_string()),
                    ],
                }
            ]),

            ProcessStage::Junos23AwaitZeroizeFinish => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::Junos23AwaitZeroizeFinish2,
                    condition: StateCondition::WaitForRegex(r"FreeBSD/[AaipP]".to_string()),
                    actions: vec![],
                }
            ]),

            ProcessStage::Junos23AwaitZeroizeFinish2 => Ok(vec![
                StateTransition {
                    target_state: ProcessStage::JunosLogin,
                    condition: StateCondition::WaitForString("login:".to_string()),
                    actions: vec![
                        Action::SendLine("root".to_string()),
                    ],
                }
            ]),

            ProcessStage::EndJob => Ok(vec![
                StateTransition {
                    condition: StateCondition::Immediate,
                    target_state: ProcessStage::SwitchDetect,
                    actions: vec![
                        Action::FinishJob,
                    ],
                }
            ]),
        }
    }
}
