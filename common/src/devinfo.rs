use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum DeviceInformation {
    SerialNumber(String),
    SoftwareVersion(String),
    Model(String),
    Vendor(String),
    AttemptedToFixFilesystemIssues,
    FailedToEnterSingleUserMode,
    ReadonlyFlash,
    SCSIErrors,
    KeptHostname,
    Aborted,
    BootLoop,
    UnableToLoadAKernel,
    AlternateImage,
    StrangeCLIPrompt,
    OSCorruption,
}

impl DeviceInformation {
    pub fn get_type(&self) -> DeviceInformationType {
        match self {
            DeviceInformation::SerialNumber(_) => DeviceInformationType::Info,
            DeviceInformation::SoftwareVersion(_) => DeviceInformationType::Info,
            DeviceInformation::Model(_) => DeviceInformationType::Info,
            DeviceInformation::Vendor(_) => DeviceInformationType::Info,
            DeviceInformation::AttemptedToFixFilesystemIssues => DeviceInformationType::Warning,
            DeviceInformation::ReadonlyFlash => DeviceInformationType::Error,
            DeviceInformation::SCSIErrors => DeviceInformationType::Error,
            DeviceInformation::KeptHostname => DeviceInformationType::Warning,
            DeviceInformation::Aborted => DeviceInformationType::Error,
            DeviceInformation::BootLoop => DeviceInformationType::Warning,
            DeviceInformation::FailedToEnterSingleUserMode => DeviceInformationType::Error,
            DeviceInformation::UnableToLoadAKernel => DeviceInformationType::Warning,
            DeviceInformation::AlternateImage => DeviceInformationType::Warning,
            DeviceInformation::StrangeCLIPrompt => DeviceInformationType::Warning,
            DeviceInformation::OSCorruption => DeviceInformationType::Error,
        }
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub enum DeviceInformationType {
    Info,
    Warning,
    Error,
}
