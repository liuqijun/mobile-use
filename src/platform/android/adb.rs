use crate::core::{MobileUseError, Result};
use serde::Serialize;
use std::process::Command;
use tracing::{debug, info};

/// Device information
#[derive(Debug, Clone, Serialize)]
pub struct DeviceInfo {
    /// Device serial/ID (e.g., "emulator-5554" or "192.168.1.100:5555")
    pub id: String,
    /// Device model (e.g., "Pixel 6")
    pub model: String,
    /// Device brand/manufacturer (e.g., "Google")
    pub brand: String,
    /// Android version (e.g., "13")
    pub android_version: String,
    /// SDK/API level (e.g., "33")
    pub sdk_version: String,
    /// Screen resolution (e.g., "1080x2400")
    pub screen_size: String,
}

/// ADB client for Android device communication
#[derive(Clone)]
pub struct AdbClient {
    device_id: Option<String>,
}

impl AdbClient {
    pub fn new(device_id: Option<String>) -> Self {
        Self { device_id }
    }

    /// Execute ADB command
    fn exec(&self, args: &[&str]) -> Result<String> {
        let mut cmd = Command::new("adb");

        if let Some(ref device) = self.device_id {
            cmd.arg("-s").arg(device);
        }

        cmd.args(args);

        debug!("Executing: adb {:?}", args);

        let output = cmd.output().map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                MobileUseError::AdbError(
                    "adb not found. Install it with: brew install android-platform-tools"
                        .to_string(),
                )
            } else {
                MobileUseError::AdbError(format!("Failed to execute adb: {}", e))
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MobileUseError::AdbError(format!(
                "ADB command failed: {}",
                stderr
            )));
        }

        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// List connected devices with basic IDs
    pub fn device_ids() -> Result<Vec<String>> {
        let output = Command::new("adb")
            .arg("devices")
            .output()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    MobileUseError::AdbError(
                        "adb not found. Install it with: brew install android-platform-tools"
                            .to_string(),
                    )
                } else {
                    MobileUseError::AdbError(format!("Failed to list devices: {}", e))
                }
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MobileUseError::AdbError(format!(
                "ADB devices command failed: {}",
                stderr
            )));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let devices: Vec<String> = stdout
            .lines()
            .skip(1) // Skip "List of devices attached"
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "device" {
                    Some(parts[0].to_string())
                } else {
                    None
                }
            })
            .collect();

        Ok(devices)
    }

    /// List connected devices with detailed information
    pub fn devices() -> Result<Vec<DeviceInfo>> {
        let device_ids = Self::device_ids()?;
        let mut devices = Vec::new();

        for id in device_ids {
            let client = AdbClient::new(Some(id.clone()));
            let info = client.get_device_info()?;
            devices.push(info);
        }

        Ok(devices)
    }

    /// Get detailed information about this device
    pub fn get_device_info(&self) -> Result<DeviceInfo> {
        let id = self
            .device_id
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // Get device properties
        let model = self
            .get_prop("ro.product.model")
            .unwrap_or_else(|_| "Unknown".to_string());
        let brand = self
            .get_prop("ro.product.brand")
            .unwrap_or_else(|_| "Unknown".to_string());
        let android_version = self
            .get_prop("ro.build.version.release")
            .unwrap_or_else(|_| "Unknown".to_string());
        let sdk_version = self
            .get_prop("ro.build.version.sdk")
            .unwrap_or_else(|_| "Unknown".to_string());

        // Get screen size
        let screen_size = self
            .get_screen_size()
            .map(|(w, h)| format!("{}x{}", w, h))
            .unwrap_or_else(|_| "Unknown".to_string());

        Ok(DeviceInfo {
            id,
            model,
            brand,
            android_version,
            sdk_version,
            screen_size,
        })
    }

    /// Get a device property
    pub fn get_prop(&self, prop: &str) -> Result<String> {
        let output = self.shell(&format!("getprop {}", prop))?;
        Ok(output.trim().to_string())
    }

    /// Get forwarded ports
    pub fn forward_list(&self) -> Result<Vec<(u16, u16)>> {
        let output = self.exec(&["forward", "--list"])?;

        let ports: Vec<(u16, u16)> = output
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 3 {
                    let local = parts[1].strip_prefix("tcp:")?.parse().ok()?;
                    let remote = parts[2].strip_prefix("tcp:")?.parse().ok()?;
                    Some((local, remote))
                } else {
                    None
                }
            })
            .collect();

        Ok(ports)
    }

    /// Forward a port
    #[allow(dead_code)]
    pub fn forward(&self, local_port: u16, remote_port: u16) -> Result<()> {
        self.exec(&[
            "forward",
            &format!("tcp:{}", local_port),
            &format!("tcp:{}", remote_port),
        ])?;
        info!("Forwarded port {} -> {}", local_port, remote_port);
        Ok(())
    }

    /// Execute shell command on device
    pub fn shell(&self, command: &str) -> Result<String> {
        self.exec(&["shell", command])
    }

    /// Tap at coordinates
    pub fn tap(&self, x: i32, y: i32) -> Result<()> {
        self.shell(&format!("input tap {} {}", x, y))?;
        Ok(())
    }

    /// Long press (swipe with same start/end)
    pub fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> Result<()> {
        self.shell(&format!(
            "input swipe {} {} {} {} {}",
            x, y, x, y, duration_ms
        ))?;
        Ok(())
    }

    /// Swipe gesture
    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        self.shell(&format!(
            "input swipe {} {} {} {} {}",
            x1, y1, x2, y2, duration_ms
        ))?;
        Ok(())
    }

    /// Input text
    pub fn input_text(&self, text: &str) -> Result<()> {
        // Escape all shell special characters
        let mut escaped = String::with_capacity(text.len() * 2);
        for c in text.chars() {
            match c {
                ' ' => escaped.push_str("%s"),
                '\'' | '"' | '\\' | '$' | '`' | '!' | '&' | '|' | ';' | '(' | ')' | '<' | '>'
                | '*' | '?' | '[' | ']' | '{' | '}' | '#' | '~' | '^' => {
                    escaped.push('\\');
                    escaped.push(c);
                }
                '\n' | '\r' | '\t' => {
                    // Skip control characters that can't be input via ADB
                }
                _ => escaped.push(c),
            }
        }
        self.shell(&format!("input text \"{}\"", escaped))?;
        Ok(())
    }

    /// Press key
    pub fn keyevent(&self, keycode: &str) -> Result<()> {
        // Validate keycode - must be numeric or alphanumeric (no special chars)
        if !keycode.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(MobileUseError::InvalidArgument(format!(
                "Invalid keycode: {}. Must be numeric or alphanumeric.",
                keycode
            )));
        }
        self.shell(&format!("input keyevent {}", keycode))?;
        Ok(())
    }

    /// Install an APK on the device
    pub fn install(&self, apk_path: &str) -> Result<()> {
        let output = self.exec(&["install", "-r", apk_path])?;
        if output.contains("Success") {
            info!("APK installed successfully");
            Ok(())
        } else {
            Err(MobileUseError::AdbError(format!(
                "APK install failed: {}",
                output.trim()
            )))
        }
    }

    /// Take screenshot and save to local path
    pub fn screenshot(&self, local_path: &str) -> Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let remote_path = format!("/sdcard/mobile-use-screenshot-{}.png", timestamp);

        // Capture screenshot on device
        self.shell(&format!("screencap -p {}", remote_path))?;

        // Pull to local
        self.exec(&["pull", &remote_path, local_path])?;

        // Clean up
        self.shell(&format!("rm {}", remote_path))?;

        info!("Screenshot saved to {}", local_path);
        Ok(())
    }

    /// Get screen size
    pub fn get_screen_size(&self) -> Result<(i32, i32)> {
        let output = self.shell("wm size")?;
        // Parse "Physical size: 1080x1920"
        let size_str = output
            .lines()
            .find(|l| l.contains("Physical size:"))
            .ok_or_else(|| MobileUseError::AdbError("Cannot get screen size".to_string()))?;

        let size_part = size_str
            .split(':')
            .nth(1)
            .ok_or_else(|| MobileUseError::AdbError("Invalid size format".to_string()))?
            .trim();

        let parts: Vec<&str> = size_part.split('x').collect();
        if parts.len() != 2 {
            return Err(MobileUseError::AdbError("Invalid size format".to_string()));
        }

        let width = parts[0]
            .parse()
            .map_err(|_| MobileUseError::AdbError("Invalid width".to_string()))?;
        let height = parts[1]
            .parse()
            .map_err(|_| MobileUseError::AdbError("Invalid height".to_string()))?;

        Ok((width, height))
    }
}

use crate::core::types::{DeviceOperator, Platform};

impl DeviceOperator for AdbClient {
    fn tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::tap(self, x, y).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn double_tap(&self, x: i32, y: i32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::tap(self, x, y).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        std::thread::sleep(std::time::Duration::from_millis(50));
        AdbClient::tap(self, x, y).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;
        Ok(())
    }

    fn long_press(&self, x: i32, y: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::long_press(self, x, y, duration_ms).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::swipe(self, x1, y1, x2, y2, duration_ms).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn input_text(&self, text: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::input_text(self, text).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn keyevent(&self, key: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::keyevent(self, key).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn screenshot(&self, local_path: &str) -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::screenshot(self, local_path).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn get_screen_size(&self) -> std::result::Result<(i32, i32), Box<dyn std::error::Error + Send + Sync>> {
        AdbClient::get_screen_size(self).map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }

    fn platform(&self) -> Platform {
        Platform::Android
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adb_client_creation() {
        let client = AdbClient::new(None);
        assert!(client.device_id.is_none());

        let client = AdbClient::new(Some("emulator-5554".to_string()));
        assert_eq!(client.device_id, Some("emulator-5554".to_string()));
    }
}
