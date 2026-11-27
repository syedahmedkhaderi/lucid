use serde::Serialize;

/// Hardware gate for the two supported downloadable local speech models.
#[derive(Debug, Clone, Serialize)]
pub struct LocalSttCapabilities {
    pub platform: String,
    pub has_gpu: bool,
    pub local_models_available: bool,
    pub reason: String,
}

pub fn current() -> LocalSttCapabilities {
    #[cfg(target_os = "macos")]
    {
        return LocalSttCapabilities {
            platform: "macOS".into(),
            has_gpu: true,
            local_models_available: true,
            reason: "Local transcription is supported on Mac using Apple hardware acceleration when available.".into(),
        };
    }

    #[cfg(target_os = "windows")]
    {
        let has_gpu = windows_has_attached_gpu();
        return LocalSttCapabilities {
            platform: "Windows".into(),
            has_gpu,
            local_models_available: has_gpu,
            reason: if has_gpu {
                "A display GPU was detected; local models are available.".into()
            } else {
                "No local GPU was detected. Use Groq Whisper for transcription.".into()
            },
        };
    }

    #[cfg(target_os = "linux")]
    {
        let has_gpu = std::process::Command::new("nvidia-smi")
            .args(["--query-gpu=name", "--format=csv,noheader"])
            .output()
            .map(|output| output.status.success() && !output.stdout.is_empty())
            .unwrap_or(false);
        return LocalSttCapabilities {
            platform: "Linux".into(),
            has_gpu,
            local_models_available: has_gpu,
            reason: if has_gpu {
                "An NVIDIA GPU was detected; local models are available.".into()
            } else {
                "No supported local GPU was detected. Use Groq Whisper for transcription.".into()
            },
        };
    }

    #[allow(unreachable_code)]
    LocalSttCapabilities {
        platform: std::env::consts::OS.into(),
        has_gpu: false,
        local_models_available: false,
        reason: "Local transcription is unavailable on this platform. Use Groq Whisper.".into(),
    }
}

#[cfg(target_os = "windows")]
fn windows_has_attached_gpu() -> bool {
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{
        EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_ATTACHED_TO_DESKTOP,
        DISPLAY_DEVICE_MIRRORING_DRIVER, DISPLAY_DEVICE_RDPUDD, DISPLAY_DEVICE_REMOTE,
    };

    for index in 0..32 {
        let mut device = DISPLAY_DEVICEW::default();
        device.cb = std::mem::size_of::<DISPLAY_DEVICEW>() as u32;
        // SAFETY: `device` is initialized with the required size and remains writable
        // for the duration of the documented Win32 enumeration call.
        let found = unsafe { EnumDisplayDevicesW(PCWSTR::null(), index, &mut device, 0) };
        if found.0 == 0 {
            break;
        }
        let flags = device.StateFlags.0;
        if flags & DISPLAY_DEVICE_ATTACHED_TO_DESKTOP.0 == 0
            || flags
                & (DISPLAY_DEVICE_MIRRORING_DRIVER.0
                    | DISPLAY_DEVICE_REMOTE.0
                    | DISPLAY_DEVICE_RDPUDD.0)
                != 0
        {
            continue;
        }
        let description = String::from_utf16_lossy(&device.DeviceString)
            .trim_matches(char::from(0))
            .to_ascii_lowercase();
        if !["basic display", "remote", "virtual", "displaylink"]
            .iter()
            .any(|marker| description.contains(marker))
        {
            return true;
        }
    }
    false
}
