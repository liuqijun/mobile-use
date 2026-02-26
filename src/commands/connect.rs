use crate::cli::OutputFormatter;
use crate::core::{MobileUseError, Result};
use crate::platform::android::AdbClient;
use tracing::info;

/// Find Flutter VM Service URL for a device
pub async fn find_flutter_vm_service(adb: &AdbClient) -> Result<String> {
    // Method 1: Check forwarded ports and try to verify they work
    let forwards = adb.forward_list()?;
    for (local, remote) in &forwards {
        let url = format!("ws://127.0.0.1:{}/ws", local);
        info!(
            "Found forwarded port {} -> {}, will try: {}",
            local, remote, url
        );
    }

    // Return first forwarded port if any exist
    if let Some((local, _)) = forwards.first() {
        if forwards.len() > 1 {
            info!(
                "Multiple forwarded ports found ({}), using first one",
                forwards.len()
            );
        }
        let url = format!("ws://127.0.0.1:{}/ws", local);
        return Ok(url);
    }

    // Method 2: Look for observatory file on device
    let output = adb.shell("cat /data/local/tmp/sky.observatory.txt 2>/dev/null || echo ''")?;
    if !output.trim().is_empty() {
        let url = output.trim().to_string();
        info!("Found observatory URL: {}", url);
        return Ok(url);
    }

    // Method 3: Parse logcat for observatory URL
    let logcat = adb.shell(
        "logcat -d -s flutter 2>/dev/null | grep -o 'http://[^[:space:]]*' | tail -1 || echo ''",
    )?;
    if !logcat.trim().is_empty() {
        let http_url = logcat.trim();
        // Convert http to ws URL - append /ws at the end
        let ws_url = format!(
            "{}/ws",
            http_url.replace("http://", "ws://").trim_end_matches('/')
        );
        info!("Found URL from logcat: {}", ws_url);
        return Ok(ws_url);
    }

    Err(MobileUseError::ConnectionFailed(
        "No Flutter app found. Make sure the app is running in debug mode.".to_string(),
    ))
}

/// List connected Android devices with detailed information
pub fn devices(output: &OutputFormatter, json_mode: bool) -> Result<()> {
    let device_list = AdbClient::devices()?;

    if device_list.is_empty() {
        output.error("No devices connected. Run 'adb devices' to check.");
        return Ok(());
    }

    if json_mode {
        // JSON output
        let json = serde_json::to_value(&device_list).unwrap_or_default();
        output.json(&json);
        return Ok(());
    }

    output.success(&format!("Found {} device(s):\n", device_list.len()));

    for (i, device) in device_list.iter().enumerate() {
        output.raw(&format!("  [{}] {}", i + 1, device.id));
        output.raw(&format!(
            "      Model:   {} ({})",
            device.model, device.brand
        ));
        output.raw(&format!(
            "      Android: {} (SDK {})",
            device.android_version, device.sdk_version
        ));
        output.raw(&format!("      Screen:  {}", device.screen_size));
        output.raw("");
    }

    output.raw("Use --device <id> or -d <id> to specify a device:");
    output.raw(&format!("  mobile-use -d {} connect", device_list[0].id));

    Ok(())
}
