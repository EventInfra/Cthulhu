use crate::AngelJob;
use color_eyre::eyre::Context;
use cthulhu_common::devinfo::DeviceInformation;
use regex::RegexBuilder;
use serde::Deserialize;
use swexpect::SwitchExpect;

#[derive(Deserialize, Clone, Debug, PartialOrd, PartialEq)]
pub enum ProcessFunction {
    FixFS,
    CaptureJunosVersion,
    CaptureChassisOutput,
    CaptureAristaVersion,
    CaptureArubaAPModel,
    CaptureArubaAPSerial,
    CaptureAristaAbootVersion,
    ArbitraryDeviceInfo,
    CaptureHPSwitchModel,
    CaptureHPSwitchSerial,
    CaptureHPOSVersionBanner,
}

impl ProcessFunction {
    pub async fn execute<T: AngelJob>(
        &self,
        job: &mut T,
        p: &mut SwitchExpect,
        data: &str,
        mat: &str,
    ) -> color_eyre::Result<()> {
        match self {
            ProcessFunction::ArbitraryDeviceInfo => {
                let br = RegexBuilder::new(r"%%%%%(?<devinfo>[^%]+)%%%%%").multi_line(true).crlf(true).build()?;
                for caps in br.captures_iter(mat) {
                    let s = caps.name("devinfo").unwrap().as_str();
                    let d: DeviceInformation = serde_json::from_str(s)?;
                    job.add_information(d).await?;
                }
                Ok(())
            }
            ProcessFunction::FixFS => {
                let bdevregex = RegexBuilder::new(r"ufs: (?<device>[/a-zA-Z0-9]+) \(.*\)$")
                    .crlf(true)
                    .multi_line(true)
                    .build()?;
                let devices: Vec<String> = bdevregex
                    .captures_iter(data)
                    .map(|c| c.name("device").unwrap().as_str().to_string())
                    .collect();

                p.exp_string("#").await.context("failed to fix fs")?;

                for dev in devices.iter() {
                    p.send_line(format!("fsck -y {dev}").as_str()).await?;
                    p.exp_string("#").await.context("failed to fix fs")?;
                }
                p.send_line("reboot").await.context("failed to fix fs")?;
                job.add_information(DeviceInformation::AttemptedToFixFilesystemIssues)
                    .await?;
                Ok(())
            }
            ProcessFunction::CaptureJunosVersion => {
                let r = RegexBuilder::new(r"(?:Model: (?<model>[a-zA-Z0-9\-]+)$)|(?:Junos: (?<version>[0-9a-zA-Z\-\.]+)$)|(?:JUNOS Base OS boot \[(?<version2>[0-9a-zA-Z\-\.]+)\]$)")
                    .multi_line(true).crlf(true).build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(model.as_str().to_string()))
                            .await?;
                    }
                    if let Some(version) = cap.name("version2") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                            .await?;
                    }
                    if let Some(version) = cap.name("version") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureChassisOutput => {
                let r = RegexBuilder::new(r"^Chassis\s+(?<serial>[A-Za-z0-9]+)\s+.*$")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureAristaVersion => {
                let r = RegexBuilder::new(r"(?:^Arista (?<model>[a-zA-Z \-0-9]+)$)|(?:^Serial number:\s+(?<serial>[A-Za-z0-9]+)$)|(?:Software image version: (?<version>[0-9\.A-Za-z]+)$)")
                    .multi_line(true).crlf(true).build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(model.as_str().to_string()))
                            .await?;
                    }
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                        .await?;
                    }
                    if let Some(version) = cap.name("version") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                        .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureAristaAbootVersion => {
                let r = RegexBuilder::new(r"(?<aboot>[\d\.-]+)$")
                    .multi_line(true).crlf(true).build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(version) = cap.name("aboot") {
                        job.add_information(DeviceInformation::BootloaderVersion(
                            version.as_str().to_string(),
                        ))
                            .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureArubaAPModel => {
                let r = RegexBuilder::new(r"^Model:\s+(?<model>[A-Za-z0-9-]+)$")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(
                            model.as_str().to_string(),
                        ))
                            .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureArubaAPSerial => {
                let r = RegexBuilder::new(r"^\s+Serial\s+:\s+(?<serial>[A-Za-z0-9-]+)$")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                            .await?;
                        break;
                    }
                }

                let r = RegexBuilder::new(r"^\s+Wired MAC\s+:\s+(?<mac>[A-Za-z0-9:]+)$")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(mac) = cap.name("mac") {
                        job.add_information(DeviceInformation::MacAddress(
                            mac.as_str().to_string(),
                        ))
                            .await?;
                        break;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureHPSwitchModel => {
                let r = RegexBuilder::new(r"(?:HP (?<model>[^\s]+) Switch|^\s*ROM Version\s*:\s*(?<bootloader>[^\s]+)|System Description\s*:\s*(?<model2>[^\s]+)|(Serial Number\s*:\s*(?<serial>[^\s]+)))")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(model) = cap.name("model") {
                        job.add_information(DeviceInformation::Model(
                            model.as_str().to_string(),
                        ))
                            .await?;
                    }
                    if let Some(model) = cap.name("model2") {
                        job.add_information(DeviceInformation::Model(
                            model.as_str().to_string(),
                        ))
                            .await?;
                    }
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                            .await?;
                    }
                    if let Some(version) = cap.name("bootloader") {
                        job.add_information(DeviceInformation::BootloaderVersion(
                            version.as_str().to_string(),
                        ))
                            .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureHPSwitchSerial => {
                let r = RegexBuilder::new(r"serialNumber=(?<serial>[^\s]+),")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(serial) = cap.name("serial") {
                        job.add_information(DeviceInformation::SerialNumber(
                            serial.as_str().to_string(),
                        ))
                            .await?;
                    }
                }
                Ok(())
            }
            ProcessFunction::CaptureHPOSVersionBanner => {
                let r = RegexBuilder::new(r"1\. Primary Software Image\s*\[(?<version>[^\s]+)\]")
                    .multi_line(true)
                    .crlf(true)
                    .build()?;
                for cap in r.captures_iter(&data) {
                    if let Some(version) = cap.name("version") {
                        job.add_information(DeviceInformation::SoftwareVersion(
                            version.as_str().to_string(),
                        ))
                            .await?;
                    }
                }
                Ok(())
            }
        }
    }
}
