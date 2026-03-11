use serde::Serialize;
use std::process::Command;
use tracing::debug;

/// iOS device information
#[derive(Debug, Clone, Serialize)]
pub struct IosDeviceInfo {
    pub id: String,
    pub name: String,
    pub ios_version: String,
    pub platform: String,
}

/// List connected iOS devices via idevice_id + ideviceinfo
pub fn list_ios_devices() -> Vec<IosDeviceInfo> {
    let output = match Command::new("idevice_id").arg("-l").output() {
        Ok(o) => o,
        Err(_) => {
            debug!("idevice_id not found, skipping iOS device discovery");
            return vec![];
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut devices = Vec::new();

    for udid in stdout.lines() {
        let udid = udid.trim();
        if udid.is_empty() {
            continue;
        }

        let name = get_device_prop(udid, "DeviceName").unwrap_or_else(|| "iPhone".to_string());
        let version =
            get_device_prop(udid, "ProductVersion").unwrap_or_else(|| "Unknown".to_string());

        devices.push(IosDeviceInfo {
            id: udid.to_string(),
            name,
            ios_version: version,
            platform: "ios".to_string(),
        });
    }

    devices
}

fn get_device_prop(udid: &str, key: &str) -> Option<String> {
    let output = Command::new("ideviceinfo")
        .args(["-u", udid, "-k", key])
        .output()
        .ok()?;
    let value = String::from_utf8_lossy(&output.stdout)
        .trim()
        .to_string();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
