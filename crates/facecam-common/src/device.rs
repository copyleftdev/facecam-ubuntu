use serde::{Deserialize, Serialize};
use std::fmt;

/// Elgato vendor ID
pub const ELGATO_VID: u16 = 0x0fd9;

/// Known Elgato camera product IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElgatoProduct {
    Facecam,
    /// Facecam on USB 2.0 — non-functional, shows "USB3-REQUIRED-FOR-FACECAM"
    FacecamUsb2Fallback,
    FacecamPro,
    FacecamMk2,
    FacecamMk2Usb2,
    CamLink4K,
    Unknown(u16),
}

impl ElgatoProduct {
    pub fn from_pid(pid: u16) -> Self {
        match pid {
            0x0078 => Self::Facecam,
            0x0077 => Self::FacecamUsb2Fallback,
            0x0079 => Self::FacecamPro,
            0x0093 => Self::FacecamMk2,
            0x0094 => Self::FacecamMk2Usb2,
            0x0066 => Self::CamLink4K,
            other => Self::Unknown(other),
        }
    }

    pub fn pid(&self) -> u16 {
        match self {
            Self::Facecam => 0x0078,
            Self::FacecamUsb2Fallback => 0x0077,
            Self::FacecamPro => 0x0079,
            Self::FacecamMk2 => 0x0093,
            Self::FacecamMk2Usb2 => 0x0094,
            Self::CamLink4K => 0x0066,
            Self::Unknown(pid) => *pid,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Facecam => "Elgato Facecam",
            Self::FacecamUsb2Fallback => "Elgato Facecam (USB2 FALLBACK — NOT FUNCTIONAL)",
            Self::FacecamPro => "Elgato Facecam Pro",
            Self::FacecamMk2 => "Elgato Facecam MK.2",
            Self::FacecamMk2Usb2 => "Elgato Facecam MK.2 (USB2)",
            Self::CamLink4K => "Elgato Cam Link 4K",
            Self::Unknown(_) => "Unknown Elgato Device",
        }
    }

    pub fn is_facecam_original(&self) -> bool {
        matches!(self, Self::Facecam)
    }

    /// Device is a Facecam stuck in USB 2.0 fallback mode
    pub fn is_usb2_fallback(&self) -> bool {
        matches!(self, Self::FacecamUsb2Fallback)
    }
}

impl fmt::Display for ElgatoProduct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (PID 0x{:04x})", self.name(), self.pid())
    }
}

/// Firmware version parsed from bcdDevice USB descriptor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
}

impl FirmwareVersion {
    pub fn from_bcd(bcd: u16) -> Self {
        Self {
            major: ((bcd >> 8) & 0xFF) as u8,
            minor: (bcd & 0xFF) as u8,
        }
    }

    /// Firmware 4.00+ has MJPEG support (empirically confirmed on 4.00,
    /// earlier research suggested 4.03 but real device shows MJPEG on 4.00)
    pub fn has_mjpeg(&self) -> bool {
        (self.major, self.minor) >= (4, 0)
    }

    /// Firmware 3.00+ added bulk/iso transfer mode selection
    pub fn has_transfer_mode_selection(&self) -> bool {
        (self.major, self.minor) >= (3, 0)
    }
}

impl fmt::Display for FirmwareVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{:02}", self.major, self.minor)
    }
}

/// Full device fingerprint collected during probe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFingerprint {
    pub product: ElgatoProduct,
    pub firmware: FirmwareVersion,
    pub serial: String,
    pub usb_bus: u8,
    pub usb_address: u8,
    pub usb_port_numbers: Vec<u8>,
    pub usb_speed: UsbSpeed,
    pub v4l2_device: Option<String>,
    pub v4l2_sysfs_path: Option<String>,
    pub driver_version: Option<String>,
    pub card_name: Option<String>,
}

impl fmt::Display for DeviceFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Device:     {}", self.product)?;
        writeln!(f, "Firmware:   {}", self.firmware)?;
        writeln!(f, "Serial:     {}", self.serial)?;
        writeln!(
            f,
            "USB:        bus {} addr {} ({})",
            self.usb_bus, self.usb_address, self.usb_speed
        )?;
        writeln!(f, "Port Path:  {:?}", self.usb_port_numbers)?;
        if let Some(ref dev) = self.v4l2_device {
            writeln!(f, "V4L2:       {}", dev)?;
        }
        if let Some(ref card) = self.card_name {
            writeln!(f, "Card:       {}", card)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
    Unknown,
}

impl fmt::Display for UsbSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low (1.5 Mbps)"),
            Self::Full => write!(f, "Full (12 Mbps)"),
            Self::High => write!(f, "High (480 Mbps)"),
            Self::Super => write!(f, "SuperSpeed (5 Gbps)"),
            Self::SuperPlus => write!(f, "SuperSpeed+ (10+ Gbps)"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl From<rusb::Speed> for UsbSpeed {
    fn from(speed: rusb::Speed) -> Self {
        match speed {
            rusb::Speed::Low => Self::Low,
            rusb::Speed::Full => Self::Full,
            rusb::Speed::High => Self::High,
            rusb::Speed::Super => Self::Super,
            rusb::Speed::SuperPlus => Self::SuperPlus,
            _ => Self::Unknown,
        }
    }
}
