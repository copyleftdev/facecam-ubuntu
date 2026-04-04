use crate::device::{ElgatoProduct, FirmwareVersion};
use crate::formats::PixelFormat;
use serde::{Deserialize, Serialize};

/// A known device quirk with its mitigation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quirk {
    pub id: &'static str,
    pub summary: &'static str,
    pub description: &'static str,
    pub affected: QuirkScope,
    pub severity: QuirkSeverity,
    pub mitigation: QuirkMitigation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuirkSeverity {
    /// Informational — no functional impact
    Info,
    /// Degraded experience but usable
    Warning,
    /// Feature broken, workaround available
    Error,
    /// Device unusable without mitigation
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuirkScope {
    /// Affects all firmware versions
    AllFirmware,
    /// Affects firmware below a version
    FirmwareBelow(FirmwareVersion),
    /// Affects specific format
    Format(PixelFormat),
    /// Affects specific product
    Product(ElgatoProduct),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QuirkMitigation {
    /// Use v4l2loopback normalization pipeline
    NormalizationPipeline,
    /// Perform USB sysfs reset before stream start
    UsbReset,
    /// Skip this format during negotiation
    SkipFormat(PixelFormat),
    /// Update firmware
    FirmwareUpdate(FirmwareVersion),
    /// Force specific pixel format
    ForceFormat(PixelFormat),
    /// No automatic mitigation available
    Manual(&'static str),
}

/// The global quirk registry — all known Facecam behaviors traceable to observations
pub fn quirk_registry() -> Vec<Quirk> {
    vec![
        Quirk {
            id: "BOGUS_NV12",
            summary: "NV12 format advertised but produces garbage frames",
            description: "The Facecam USB descriptor advertises NV12 (Y/CbCr 4:2:0) as a \
                supported format, but streaming in NV12 produces green or empty frames. \
                Only YUYV produces valid uncompressed output.",
            affected: QuirkScope::Format(PixelFormat::Nv12),
            severity: QuirkSeverity::Error,
            mitigation: QuirkMitigation::SkipFormat(PixelFormat::Nv12),
        },
        Quirk {
            id: "BOGUS_YU12",
            summary: "YU12 format advertised but produces garbage frames",
            description: "The Facecam USB descriptor advertises YU12 (Planar YUV 4:2:0) as a \
                supported format, but streaming in YU12 produces green or empty frames. \
                Only YUYV produces valid uncompressed output.",
            affected: QuirkScope::Format(PixelFormat::Yu12),
            severity: QuirkSeverity::Error,
            mitigation: QuirkMitigation::SkipFormat(PixelFormat::Yu12),
        },
        Quirk {
            id: "OPEN_CLOSE_LOCKUP",
            summary: "Device locks up after consumer close/reopen cycle",
            description: "After the first application closes the V4L2 device, subsequent opens \
                fail with EBUSY or produce no frames. The device requires a USB reset \
                (sysfs authorized flag cycle) to recover. The v4l2loopback normalization \
                pipeline mitigates this by keeping a single long-lived producer.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Critical,
            mitigation: QuirkMitigation::NormalizationPipeline,
        },
        Quirk {
            id: "STARTUP_UNRELIABILITY",
            summary: "~50% failure rate on initial stream start",
            description: "The Facecam fails to initialize the video stream approximately half \
                the time on first open. A USB reset followed by retry resolves this. \
                The daemon must implement retry-with-reset logic.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Error,
            mitigation: QuirkMitigation::UsbReset,
        },
        Quirk {
            id: "NO_MJPEG_OLD_FW",
            summary: "No MJPEG support on firmware < 4.03",
            description: "Firmware versions below 4.03 only support uncompressed formats. \
                Chromium-based browsers require MJPEG or cannot negotiate the camera. \
                The v4l2loopback pipeline resolves this by presenting a normalized output.",
            affected: QuirkScope::FirmwareBelow(FirmwareVersion { major: 4, minor: 3 }),
            severity: QuirkSeverity::Warning,
            mitigation: QuirkMitigation::NormalizationPipeline,
        },
        Quirk {
            id: "CHROMIUM_FORMAT_REJECT",
            summary: "Chromium rejects devices with both CAPTURE and OUTPUT caps",
            description: "Chromium-based browsers refuse to use V4L2 devices that report both \
                V4L2_CAP_VIDEO_CAPTURE and V4L2_CAP_VIDEO_OUTPUT in their capabilities. \
                v4l2loopback with exclusive_caps=1 resolves this.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Error,
            mitigation: QuirkMitigation::NormalizationPipeline,
        },
        Quirk {
            id: "USB2_FALLBACK_MODE",
            summary: "Facecam presents PID 0x0077 on USB 2.0 with no video capability",
            description: "When connected to a USB 2.0 port, the Facecam enumerates with \
                PID 0x0077 instead of 0x0078 and a product string of \
                'USB3-REQUIRED-FOR-FACECAM'. No UVC interface is exposed and no \
                /dev/video node is created. The device must be moved to a USB 3.0 port.",
            affected: QuirkScope::Product(ElgatoProduct::FacecamUsb2Fallback),
            severity: QuirkSeverity::Critical,
            mitigation: QuirkMitigation::Manual("Move the camera to a USB 3.0 (blue) port"),
        },
        Quirk {
            id: "USB3_REQUIRED",
            summary: "Device requires USB 3.0 SuperSpeed",
            description: "The Facecam will not enumerate on USB 2.0 ports. YUYV 1080p60 \
                requires ~249 MB/s bandwidth which exceeds USB 2.0 High-Speed (60 MB/s). \
                USB topology must be validated during probe.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Critical,
            mitigation: QuirkMitigation::Manual("Connect to a USB 3.0 port directly, avoid hubs"),
        },
        Quirk {
            id: "BANDWIDTH_STARVATION",
            summary: "USB hub/dock sharing can cause frame drops or freezing",
            description: "At ~249 MB/s for YUYV 1080p60, the Facecam uses a significant \
                fraction of USB 3.0 bandwidth. Sharing a controller with other high-bandwidth \
                devices causes instability. MJPEG mode (firmware 4.03+) reduces bandwidth.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Warning,
            mitigation: QuirkMitigation::ForceFormat(PixelFormat::Mjpeg),
        },
        Quirk {
            id: "YUYV_UYVY_AMBIGUITY",
            summary: "Wire format may be UYVY despite V4L2 reporting YUYV",
            description: "Community workarounds use uyvy422 as ffmpeg input format even though \
                v4l2-ctl reports YUYV. The actual byte order on the wire should be verified \
                by inspecting frame data. If the first two bytes of a white pixel are \
                0x80 0xEB, it's YUYV; if 0xEB 0x80, it's UYVY.",
            affected: QuirkScope::AllFirmware,
            severity: QuirkSeverity::Info,
            mitigation: QuirkMitigation::Manual("Verify empirically via frame byte inspection"),
        },
    ]
}

/// Check which quirks apply to a specific device
pub fn applicable_quirks(product: ElgatoProduct, firmware: FirmwareVersion) -> Vec<&'static Quirk> {
    // Leak the vec so we get 'static references — acceptable for a small registry
    let registry: &'static Vec<Quirk> = Box::leak(Box::new(quirk_registry()));

    registry
        .iter()
        .filter(|q| match &q.affected {
            QuirkScope::AllFirmware => product.is_facecam_original(),
            QuirkScope::FirmwareBelow(v) => product.is_facecam_original() && firmware < *v,
            QuirkScope::Format(_) => product.is_facecam_original(),
            QuirkScope::Product(p) => *p == product,
        })
        .collect()
}
