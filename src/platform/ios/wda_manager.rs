use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info};

/// Helper to wrap command errors with install hints
fn cmd_not_found_hint(tool: &str, err: std::io::Error) -> Box<dyn std::error::Error> {
    if err.kind() == std::io::ErrorKind::NotFound {
        match tool {
            "git" => "git not found. Install it with: xcode-select --install".into(),
            "xcodebuild" => {
                "xcodebuild not found. Install Xcode from the App Store, then run: xcode-select --install".into()
            }
            "iproxy" => {
                "iproxy not found. Install it with: brew install libimobiledevice".into()
            }
            _ => format!("{} not found", tool).into(),
        }
    } else {
        format!("Failed to run {}: {}", tool, err).into()
    }
}

/// Replace Facebook bundle IDs in WDA project to avoid signing conflicts
/// with personal development teams that can't register com.facebook.* identifiers
fn patch_bundle_ids(wda_dir: &std::path::Path, team_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let pbxproj = wda_dir.join("WebDriverAgent.xcodeproj").join("project.pbxproj");
    if !pbxproj.exists() {
        return Err("project.pbxproj not found in WDA directory".into());
    }

    let content = std::fs::read_to_string(&pbxproj)?;
    let prefix = format!("com.mobileuse.{}", team_id.to_lowercase());
    let patched = content.replace("com.facebook", &prefix);

    if patched != content {
        std::fs::write(&pbxproj, &patched)?;
        info!("Patched WDA bundle IDs: com.facebook -> {}", prefix);
    } else {
        debug!("WDA bundle IDs already patched");
    }

    Ok(())
}

/// Get the WDA project directory (cloned repo)
pub fn wda_project_dir() -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("WebDriverAgent")
}

/// Clone or update WebDriverAgent repository
pub fn ensure_wda_repo() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let wda_dir = wda_project_dir();

    if wda_dir.join("WebDriverAgent.xcodeproj").exists() {
        info!("WDA repo already exists at {:?}, updating...", wda_dir);
        let status = Command::new("git")
            .args(["pull", "--ff-only"])
            .current_dir(&wda_dir)
            .status()
            .map_err(|e| cmd_not_found_hint("git", e))?;
        if !status.success() {
            info!("Git pull failed, continuing with existing version");
        }
    } else {
        info!("Cloning WebDriverAgent to {:?}", wda_dir);
        if let Some(parent) = wda_dir.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let status = Command::new("git")
            .args([
                "clone",
                "https://github.com/appium/WebDriverAgent.git",
                &wda_dir.to_string_lossy(),
            ])
            .status()
            .map_err(|e| cmd_not_found_hint("git", e))?;
        if !status.success() {
            return Err("Failed to clone WebDriverAgent".into());
        }
    }

    Ok(wda_dir)
}

/// Build and install WDA on a real device
pub fn build_and_install_wda(
    device_id: &str,
    team_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let wda_dir = ensure_wda_repo()?;

    // Replace Facebook bundle IDs with unique ones to avoid signing conflicts
    patch_bundle_ids(&wda_dir, team_id)?;

    info!("Building WebDriverAgent for device {}...", device_id);
    info!("Using development team: {}", team_id);

    let status = Command::new("xcodebuild")
        .args([
            "build-for-testing",
            "-project",
            "WebDriverAgent.xcodeproj",
            "-scheme",
            "WebDriverAgentRunner",
            "-destination",
            &format!("id={}", device_id),
            "-derivedDataPath",
            "build",
            "-allowProvisioningUpdates",
            &format!("DEVELOPMENT_TEAM={}", team_id),
            "CODE_SIGNING_ALLOWED=YES",
        ])
        .current_dir(&wda_dir)
        .status()
        .map_err(|e| cmd_not_found_hint("xcodebuild", e))?;

    if !status.success() {
        return Err(
            "xcodebuild failed. Make sure your Apple Developer Team ID is correct.".into(),
        );
    }

    info!("WDA built successfully");
    Ok(())
}

/// Launch WDA on device using xcodebuild test
/// Returns the WDA port (default 8100)
pub fn launch_wda(device_id: &str, team_id: &str, port: u16) -> Result<u16, Box<dyn std::error::Error>> {
    let wda_dir = wda_project_dir();

    if !wda_dir.join("build").exists() {
        return Err("WDA not built. Run 'mobile-use setup-ios' first.".into());
    }

    // Check if WDA is already running
    if is_wda_running(port) {
        info!("WDA is already running on port {}", port);
        return Ok(port);
    }

    info!("Launching WDA on device {}...", device_id);

    // Start xcodebuild test in background
    let child = Command::new("xcodebuild")
        .args([
            "test-without-building",
            "-project",
            "WebDriverAgent.xcodeproj",
            "-scheme",
            "WebDriverAgentRunner",
            "-destination",
            &format!("id={}", device_id),
            "-derivedDataPath",
            "build",
            &format!("DEVELOPMENT_TEAM={}", team_id),
        ])
        .current_dir(&wda_dir)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| cmd_not_found_hint("xcodebuild", e))?;

    debug!("WDA xcodebuild test process started (PID: {})", child.id());

    // Save PID for cleanup
    let pid_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("wda.pid");
    std::fs::write(&pid_path, child.id().to_string())?;

    // Start iproxy for port forwarding
    start_iproxy(device_id, port)?;

    // Wait for WDA to be ready
    wait_for_wda(port)?;

    Ok(port)
}

/// Start iproxy for port forwarding
fn start_iproxy(device_id: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    info!(
        "Starting iproxy {}:{} for device {}",
        port, port, device_id
    );

    let child = Command::new("iproxy")
        .args([&port.to_string(), &port.to_string(), "-u", device_id])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| cmd_not_found_hint("iproxy", e))?;

    // Save PID for cleanup
    let pid_path = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join("iproxy.pid");
    std::fs::write(&pid_path, child.id().to_string())?;

    // Brief wait for iproxy to start
    std::thread::sleep(std::time::Duration::from_millis(500));
    Ok(())
}

/// Check if WDA is already running on given port
fn is_wda_running(port: u16) -> bool {
    let url = format!("http://127.0.0.1:{}/status", port);
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .no_proxy()
        .build()
        .ok()
        .and_then(|c| c.get(&url).send().ok())
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

/// Wait for WDA to respond on given port
fn wait_for_wda(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:{}/status", port);
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .no_proxy()
        .build()?;

    info!("Waiting for WDA to be ready on port {}...", port);

    for i in 0..60 {
        match client.get(&url).send() {
            Ok(resp) if resp.status().is_success() => {
                info!("WDA is ready (attempt {})", i + 1);
                return Ok(());
            }
            _ => {
                debug!("WDA not ready yet (attempt {})", i + 1);
                std::thread::sleep(std::time::Duration::from_secs(2));
            }
        }
    }

    Err("WDA did not start within 120 seconds".into())
}

/// Kill running WDA and iproxy processes
pub fn stop_wda() {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use");

    for name in ["wda.pid", "iproxy.pid"] {
        let pid_path = cache_dir.join(name);
        if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let _ = Command::new("kill")
                    .args(["-TERM", &pid.to_string()])
                    .status();
                info!("Stopped {} (PID {})", name.replace(".pid", ""), pid);
            }
            let _ = std::fs::remove_file(&pid_path);
        }
    }
}
