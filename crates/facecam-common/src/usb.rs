use crate::device::{DeviceFingerprint, ElgatoProduct, FirmwareVersion, UsbSpeed, ELGATO_VID};
use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::info;

/// Discover Elgato cameras via libusb
pub fn enumerate_elgato_devices() -> Result<Vec<DeviceFingerprint>> {
    let mut devices = Vec::new();

    for device in rusb::devices()?.iter() {
        let desc = device.device_descriptor()?;
        if desc.vendor_id() != ELGATO_VID {
            continue;
        }

        let product = ElgatoProduct::from_pid(desc.product_id());
        let ver = desc.device_version();
        // bcdDevice 0x0409 -> major=4, minor=0, sub_minor=9 in rusb
        // We need major as the firmware major, and (minor*10 + sub_minor) as firmware minor
        let bcd = ((ver.major() as u16) << 8) | ((ver.minor() as u16) * 10 + ver.sub_minor() as u16);
        let speed: UsbSpeed = device.speed().into();

        let firmware = FirmwareVersion::from_bcd(bcd);

        // Skip USB descriptor reads for devices in USB2 fallback mode —
        // they hang on open() because the device is non-functional.
        let serial = if product.is_usb2_fallback() {
            String::new()
        } else {
            match device.open() {
                Ok(handle) => {
                    handle
                        .read_string_descriptor_ascii(desc.serial_number_string_index().unwrap_or(0))
                        .unwrap_or_default()
                }
                Err(_) => String::new(),
            }
        };

        let port_numbers = device.port_numbers().unwrap_or_default();

        let fingerprint = DeviceFingerprint {
            product,
            firmware,
            serial,
            usb_bus: device.bus_number(),
            usb_address: device.address(),
            usb_port_numbers: port_numbers,
            usb_speed: speed,
            v4l2_device: None,
            v4l2_sysfs_path: None,
            driver_version: None,
            card_name: None,
        };

        info!(
            product = %fingerprint.product,
            firmware = %fingerprint.firmware,
            bus = fingerprint.usb_bus,
            addr = fingerprint.usb_address,
            speed = %fingerprint.usb_speed,
            "Found Elgato device"
        );

        devices.push(fingerprint);
    }

    Ok(devices)
}

/// Find the sysfs path for a USB device by bus and address
pub fn find_usb_sysfs_path(bus: u8, addr: u8) -> Result<Option<PathBuf>> {
    let sysfs_base = Path::new("/sys/bus/usb/devices");
    if !sysfs_base.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(sysfs_base)? {
        let entry = entry?;
        let path = entry.path();

        let busnum_path = path.join("busnum");
        let devnum_path = path.join("devnum");

        if busnum_path.exists() && devnum_path.exists() {
            let busnum: u8 = fs::read_to_string(&busnum_path)?.trim().parse().unwrap_or(0);
            let devnum: u8 = fs::read_to_string(&devnum_path)?.trim().parse().unwrap_or(0);

            if busnum == bus && devnum == addr {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}

/// Find sysfs path for Elgato Facecam by VID:PID
pub fn find_facecam_sysfs_path() -> Result<Option<PathBuf>> {
    let sysfs_base = Path::new("/sys/bus/usb/devices");
    if !sysfs_base.exists() {
        return Ok(None);
    }

    for entry in fs::read_dir(sysfs_base)? {
        let entry = entry?;
        let path = entry.path();

        let vid_path = path.join("idVendor");
        let pid_path = path.join("idProduct");

        if vid_path.exists() && pid_path.exists() {
            let vid = fs::read_to_string(&vid_path)?.trim().to_string();
            let pid = fs::read_to_string(&pid_path)?.trim().to_string();

            if vid == format!("{:04x}", ELGATO_VID)
                && pid == format!("{:04x}", ElgatoProduct::Facecam.pid())
            {
                return Ok(Some(path));
            }
        }
    }

    Ok(None)
}

/// Find the V4L2 device node associated with a USB device sysfs path
pub fn find_v4l2_device_for_usb(usb_sysfs: &Path) -> Result<Option<String>> {
    // Walk the USB device tree looking for video4linux subdirectories
    find_v4l2_node_recursive(usb_sysfs)
}

fn find_v4l2_node_recursive(path: &Path) -> Result<Option<String>> {
    let v4l_path = path.join("video4linux");
    if v4l_path.exists() {
        for entry in fs::read_dir(&v4l_path)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("video") {
                // Check index — we want index 0 (primary capture, not metadata)
                let index_path = entry.path().join("index");
                if index_path.exists() {
                    let index: u32 = fs::read_to_string(&index_path)?
                        .trim()
                        .parse()
                        .unwrap_or(u32::MAX);
                    if index == 0 {
                        return Ok(Some(format!("/dev/{}", name)));
                    }
                } else {
                    return Ok(Some(format!("/dev/{}", name)));
                }
            }
        }
    }

    // Recurse into subdirectories (USB interfaces are children)
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries {
            let entry = entry?;
            let child = entry.path();
            if child.is_dir() {
                if let Some(dev) = find_v4l2_node_recursive(&child)? {
                    return Ok(Some(dev));
                }
            }
        }
    }

    Ok(None)
}

/// Read USB topology details for diagnostics
pub fn read_usb_topology(sysfs_path: &Path) -> Result<UsbTopology> {
    let read_file = |name: &str| -> Option<String> {
        fs::read_to_string(sysfs_path.join(name))
            .ok()
            .map(|s| s.trim().to_string())
    };

    Ok(UsbTopology {
        sysfs_path: sysfs_path.to_path_buf(),
        busnum: read_file("busnum").and_then(|s| s.parse().ok()),
        devnum: read_file("devnum").and_then(|s| s.parse().ok()),
        speed: read_file("speed"),
        version: read_file("version"),
        maxchild: read_file("maxchild").and_then(|s| s.parse().ok()),
        authorized: read_file("authorized").and_then(|s| s.parse().ok()),
        manufacturer: read_file("manufacturer"),
        product_name: read_file("product"),
        bcd_device: read_file("bcdDevice"),
        configuration: read_file("configuration"),
    })
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsbTopology {
    pub sysfs_path: PathBuf,
    pub busnum: Option<u8>,
    pub devnum: Option<u8>,
    pub speed: Option<String>,
    pub version: Option<String>,
    pub maxchild: Option<u8>,
    pub authorized: Option<u8>,
    pub manufacturer: Option<String>,
    pub product_name: Option<String>,
    pub bcd_device: Option<String>,
    pub configuration: Option<String>,
}
