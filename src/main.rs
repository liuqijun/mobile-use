mod cli;
mod commands;
mod core;
mod daemon;
mod platform;

use anyhow::Result;
use cli::{Commands, DaemonCommands, FlutterCommands, OutputFormatter};
use core::{ActionResult, Direction, ElementRef, RefMap};
use daemon::{
    get_pid_path, get_socket_path, DaemonClient, DaemonRequest, DaemonResponse, DaemonServer,
    DeviceAction,
};
use platform::android::AdbClient;
use platform::android::gradle;
use platform::android::uiautomator;
use platform::flutter::{
    match_styles_to_nodes, parse_render_tree, parse_semantics_tree, FlutterEvent,
    FlutterProcessManager,
};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde_json::json;
use tokio::time::Duration;
use tracing_subscriber::EnvFilter;

/// Maximum number of characters to delete when clearing a text field
const MAX_CLEAR_DELETE_PRESSES: u32 = 50;

/// Default swipe distance in pixels
const DEFAULT_SWIPE_DISTANCE: i32 = 500;

/// Detected project type
#[derive(Debug, Clone, PartialEq)]
enum ProjectType {
    FlutterAndroid,
    NativeAndroid,
    Unknown,
}

/// Detect project type from current directory and arguments
fn detect_project_type(apk: &Option<String>, package: &Option<String>) -> ProjectType {
    if apk.is_some() || package.is_some() {
        return ProjectType::NativeAndroid;
    }

    let has_pubspec = std::path::Path::new("pubspec.yaml").exists();
    let has_android = std::path::Path::new("android").exists();

    if has_pubspec && has_android {
        return ProjectType::FlutterAndroid;
    }

    // Check for native Android Gradle project at root level
    let has_gradle = std::path::Path::new("build.gradle").exists()
        || std::path::Path::new("build.gradle.kts").exists()
        || std::path::Path::new("settings.gradle.kts").exists()
        || std::path::Path::new("settings.gradle").exists();

    if has_gradle && !has_pubspec {
        return ProjectType::NativeAndroid;
    }

    if has_android {
        ProjectType::NativeAndroid
    } else {
        ProjectType::Unknown
    }
}

/// Get the PID file path for a running session
fn get_run_pid_path(session: &str) -> PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use")
        .join(format!("run-{}.pid", session))
}

/// Perform a full state reset: kill all processes, remove all state files
async fn reset_all(output: &OutputFormatter) {
    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("mobile-use");

    // 1. Kill all run wrapper processes via PID files
    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with("run-") && name.ends_with(".pid") {
                    if let Ok(pid_str) = std::fs::read_to_string(&path) {
                        if let Ok(pid) = pid_str.trim().parse::<u32>() {
                            let _ = std::process::Command::new("kill")
                                .args(["-TERM", &pid.to_string()])
                                .status();
                            output.info(&format!("Sent stop signal to run process {}", pid));
                        }
                    }
                    let _ = std::fs::remove_file(&path);
                }
            }
        }
    }

    // 2. Kill orphaned flutter processes (exact command match)
    let _ = std::process::Command::new("pkill")
        .args(["-xf", "flutter run --machine"])
        .status();

    // 3. Stop daemon gracefully (if reachable)
    if DaemonClient::is_daemon_running() {
        if let Ok(mut client) = DaemonClient::connect().await {
            let _ = client.request(DaemonRequest::Shutdown).await;
            output.info("Daemon stopped");
        }
        // Give daemon time to clean up before force kill
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    // 4. Kill daemon process via PID file (if graceful shutdown failed)
    let daemon_pid_path = cache_dir.join("daemon.pid");
    if daemon_pid_path.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(&daemon_pid_path) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                let _ = std::process::Command::new("kill")
                    .args(["-9", &pid.to_string()])
                    .status();
            }
        }
        let _ = std::fs::remove_file(&daemon_pid_path);
    }

    // 5. Remove socket file
    let socket_path = cache_dir.join("daemon.sock");
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }

    // 6. Remove legacy session files
    let sessions_dir = cache_dir.join("sessions");
    if sessions_dir.exists() {
        let _ = std::fs::remove_dir_all(&sessions_dir);
    }

    // Brief wait for processes to terminate
    tokio::time::sleep(Duration::from_millis(500)).await;

    output.success("All state reset. Run 'mobile-use run' to start fresh.");
}

/// Helper macro to handle daemon responses
macro_rules! handle_response {
    ($output:expr, $response:expr, $success_handler:expr) => {
        match $response {
            Ok(DaemonResponse::Ok { data }) => $success_handler(data),
            Ok(DaemonResponse::Error { message }) => {
                $output.error(&message);
                Err(anyhow::anyhow!(message))
            }
            Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                $output.error("Unexpected HasFlutterProcess response");
                Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
            }
            Err(e) => {
                $output.error(&e.to_string());
                Err(e.into())
            }
        }
    };
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let args = cli::parse();
    let output = OutputFormatter::new(args.json);

    // Handle daemon commands first (don't need session)
    match &args.command {
        // Daemon start - run the daemon server
        Commands::Daemon(DaemonCommands::Start) => {
            let server = DaemonServer::new();
            if let Err(e) = server.run().await {
                output.error(&format!("Daemon server error: {}", e));
                std::process::exit(1);
            }
            return Ok(());
        }

        // Daemon stop (or Stop alias) - send shutdown request
        Commands::Daemon(DaemonCommands::Stop) | Commands::Stop => {
            if !DaemonClient::is_daemon_running() {
                output.info("Daemon is not running");
                return Ok(());
            }

            match DaemonClient::connect().await {
                Ok(mut client) => {
                    let response = client.request(DaemonRequest::Shutdown).await;
                    match response {
                        Ok(DaemonResponse::Ok { .. }) => {
                            output.success("Daemon stopped");
                        }
                        Ok(DaemonResponse::Error { message }) => {
                            output.error(&format!("Failed to stop daemon: {}", message));
                            std::process::exit(1);
                        }
                        Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                            output.error("Unexpected HasFlutterProcess response");
                            std::process::exit(1);
                        }
                        Err(e) => {
                            output.error(&format!("Failed to stop daemon: {}", e));
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    output.error(&format!("Failed to connect to daemon: {}", e));
                    std::process::exit(1);
                }
            }
            return Ok(());
        }

        // Daemon status - check if daemon is running
        Commands::Daemon(DaemonCommands::Status) => {
            let socket_path = get_socket_path();
            let pid_path = get_pid_path();
            let running = DaemonClient::is_daemon_running();

            if args.json {
                output.json(&serde_json::json!({
                    "running": running,
                    "socket_path": socket_path.to_string_lossy(),
                    "pid_path": pid_path.to_string_lossy()
                }));
            } else if running {
                output.info(&format!(
                    "Daemon is running\n  Socket: {}\n  PID file: {}",
                    socket_path.display(),
                    pid_path.display()
                ));
            } else {
                output.info(&format!(
                    "Daemon is not running\n  Socket: {}\n  PID file: {}",
                    socket_path.display(),
                    pid_path.display()
                ));
            }
            return Ok(());
        }

        // Devices - doesn't need daemon
        Commands::Devices => {
            if let Err(e) = commands::devices(&output, args.json) {
                output.error(&e.to_string());
                std::process::exit(1);
            }
            // Also list iOS devices
            let ios_devices = platform::ios::list_ios_devices();
            if !ios_devices.is_empty() {
                output.raw(&format!("\niOS Devices ({}):", ios_devices.len()));
                for device in &ios_devices {
                    output.raw(&format!("  {}", device.id));
                    output.raw(&format!("      Name:    {}", device.name));
                    output.raw(&format!("      iOS:     {}", device.ios_version));
                    output.raw("");
                }
            }
            return Ok(());
        }

        // Setup iOS - build and install WDA (doesn't need daemon)
        Commands::SetupIos { team_id } => {
            let device_id = args.device.unwrap_or_else(|| {
                match std::process::Command::new("idevice_id")
                    .arg("-l")
                    .output()
                {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        stdout.lines().next().unwrap_or("").trim().to_string()
                    }
                    Err(_) => String::new(),
                }
            });

            if device_id.is_empty() {
                output.error(
                    "No iOS device found. Connect a device via USB and try again.",
                );
                std::process::exit(1);
            }

            output.info(&format!("Setting up WDA for device: {}", device_id));

            if let Err(e) = platform::ios::ensure_wda_repo() {
                output.error(&format!("Failed to get WDA: {}", e));
                std::process::exit(1);
            }

            match platform::ios::build_and_install_wda(&device_id, &team_id) {
                Ok(()) => {
                    output.success("WebDriverAgent installed successfully!");
                    output.info("Now run: mobile-use connect-ios --team-id YOUR_TEAM_ID");
                }
                Err(e) => {
                    output.error(&format!("Setup failed: {}", e));
                    std::process::exit(1);
                }
            }
            return Ok(());
        }

        // Quit - stop a running flutter process
        Commands::Quit { all } => {
            if *all {
                reset_all(&output).await;
                return Ok(());
            }
            let pid_path = get_run_pid_path(&args.session);
            if !pid_path.exists() {
                output.error(&format!(
                    "No running process found for session '{}'",
                    args.session
                ));
                std::process::exit(1);
            }

            let pid_str = match std::fs::read_to_string(&pid_path) {
                Ok(s) => s,
                Err(e) => {
                    output.error(&format!("Failed to read PID file: {}", e));
                    std::process::exit(1);
                }
            };
            let pid: u32 = match pid_str.trim().parse() {
                Ok(p) => p,
                Err(_) => {
                    output.error("Invalid PID file content");
                    let _ = std::fs::remove_file(&pid_path);
                    std::process::exit(1);
                }
            };

            // Send SIGTERM to the run process
            let status = std::process::Command::new("kill")
                .args(["-TERM", &pid.to_string()])
                .status();

            match status {
                Ok(s) if s.success() => {
                    output.info(&format!("Stopping process {}...", pid));
                    // Wait for process to exit (PID file removed by run command)
                    let mut waited = 0;
                    while pid_path.exists() && waited < 10 {
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        waited += 1;
                    }
                    if pid_path.exists() {
                        // Force kill
                        let _ = std::process::Command::new("kill")
                            .args(["-9", &pid.to_string()])
                            .status();
                        let _ = std::fs::remove_file(&pid_path);
                        output.success("Process force stopped");
                    } else {
                        output.success("Process stopped");
                    }
                }
                _ => {
                    // Process already dead, clean up PID file
                    let _ = std::fs::remove_file(&pid_path);
                    output.info("Process already stopped");
                }
            }
            return Ok(());
        }

        _ => {}
    }

    // For other commands, connect to daemon and send appropriate requests
    let mut client = match DaemonClient::connect().await {
        Ok(c) => c,
        Err(e) => {
            output.error(&format!("Failed to connect to daemon: {}", e));
            std::process::exit(1);
        }
    };

    let session_name = args.session.clone();
    let device = args.device.clone();

    let result = match args.command {
        Commands::Connect { url, port, package } => {
            if let Some(pkg) = package {
                // Android mode - create ADB-only session
                let request = DaemonRequest::ConnectAndroid {
                    session: session_name.clone(),
                    device: device.clone(),
                    package: pkg.clone(),
                };
                match client.request(request).await {
                    Ok(DaemonResponse::Ok { data }) => {
                        if let Some(data) = data {
                            if args.json {
                                output.json(&data);
                            } else {
                                output.success(&format!("Connected to Android package: {}", pkg));
                            }
                        } else {
                            output.success(&format!("Connected to Android package: {}", pkg));
                        }
                        Ok(())
                    }
                    Ok(DaemonResponse::Error { message }) => {
                        output.error(&message);
                        Err(anyhow::anyhow!(message))
                    }
                    Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                        output.error("Unexpected HasFlutterProcess response");
                        Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
                    }
                    Err(e) => {
                        output.error(&e.to_string());
                        Err(e.into())
                    }
                }
            } else {
                // Flutter mode - existing logic
                let request = DaemonRequest::Connect {
                    session: session_name,
                    device,
                    url,
                    port,
                };
                match client.request(request).await {
                    Ok(DaemonResponse::Ok { data }) => {
                        if let Some(data) = data {
                            if args.json {
                                output.json(&data);
                            } else {
                                let url = data["url"].as_str().unwrap_or("unknown");
                                output.success(&format!("Connected to {}", url));
                            }
                        } else {
                            output.success("Connected");
                        }
                        Ok(())
                    }
                    Ok(DaemonResponse::Error { message }) => {
                        output.error(&message);
                        Err(anyhow::anyhow!(message))
                    }
                    Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                        output.error("Unexpected HasFlutterProcess response");
                        Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
                    }
                    Err(e) => {
                        output.error(&e.to_string());
                        Err(e.into())
                    }
                }
            }
        }

        Commands::ConnectIos { team_id, port: _ } => {
            let device_id = device.clone().unwrap_or_else(|| {
                match std::process::Command::new("idevice_id")
                    .arg("-l")
                    .output()
                {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout);
                        stdout.lines().next().unwrap_or("").trim().to_string()
                    }
                    Err(_) => String::new(),
                }
            });

            if device_id.is_empty() {
                output.error("No iOS device found.");
                std::process::exit(1);
            }

            output.info(&format!("Launching WDA on device {}...", device_id));

            let wda_port = match platform::ios::launch_wda(&device_id, &team_id) {
                Ok(p) => p,
                Err(e) => {
                    output.error(&format!("Failed to launch WDA: {}", e));
                    output.info("Run 'mobile-use setup-ios --team-id YOUR_TEAM_ID' first.");
                    std::process::exit(1);
                }
            };

            // Send ConnectIos request to daemon (daemon creates the WdaClient)
            let request = DaemonRequest::ConnectIos {
                session: session_name.clone(),
                device: Some(device_id.clone()),
                wda_port,
            };
            match client.request(request).await {
                Ok(DaemonResponse::Ok { .. }) => {
                    output.success(&format!("Connected to iOS device: {}", device_id));
                }
                Ok(DaemonResponse::Error { message }) => {
                    output.error(&message);
                    std::process::exit(1);
                }
                _ => {
                    output.error("Unexpected response");
                    std::process::exit(1);
                }
            }
            Ok(())
        }

        Commands::Disconnect => {
            let request = DaemonRequest::Disconnect {
                session: session_name,
            };
            match client.request(request).await {
                Ok(DaemonResponse::Ok { .. }) => {
                    output.success("Disconnected");
                    Ok(())
                }
                Ok(DaemonResponse::Error { message }) => {
                    output.error(&message);
                    Err(anyhow::anyhow!(message))
                }
                Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                    output.error("Unexpected HasFlutterProcess response");
                    Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
                }
                Err(e) => {
                    output.error(&e.to_string());
                    Err(e.into())
                }
            }
        }

        Commands::Info => {
            let request = DaemonRequest::Info {
                session: session_name,
            };
            match client.request(request).await {
                Ok(DaemonResponse::Ok { data }) => {
                    if let Some(data) = data {
                        if args.json {
                            output.json(&data);
                        } else {
                            let session = data["session"].as_str().unwrap_or("unknown");
                            let connected = data["connected"].as_bool().unwrap_or(false);
                            let vm_url = data["vm_url"].as_str().unwrap_or("not connected");
                            output.info(&format!(
                                "Session: {}\nConnected: {}\nVM URL: {}",
                                session, connected, vm_url
                            ));
                        }
                    } else {
                        output.info("No session info available");
                    }
                    Ok(())
                }
                Ok(DaemonResponse::Error { message }) => {
                    output.error(&message);
                    Err(anyhow::anyhow!(message))
                }
                Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                    output.error("Unexpected HasFlutterProcess response");
                    Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
                }
                Err(e) => {
                    output.error(&e.to_string());
                    Err(e.into())
                }
            }
        }

        // Daemon commands already handled above
        Commands::Daemon(_)
        | Commands::Stop
        | Commands::Devices
        | Commands::Quit { .. }
        | Commands::SetupIos { .. } => {
            unreachable!("Already handled above")
        }

        // Elements command - get element tree with styles
        Commands::Elements { interactive } => {
            // Check session mode: if Android mode, use UIAutomator
            let info_request = DaemonRequest::Info {
                session: session_name.clone(),
            };
            let session_info = client.request(info_request).await;
            let is_android = match &session_info {
                Ok(DaemonResponse::Ok { data: Some(data) }) => {
                    data.get("mode").and_then(|v| v.as_str()) == Some("android")
                }
                _ => false,
            };

            // Check for iOS mode
            let is_ios = match &session_info {
                Ok(DaemonResponse::Ok { data: Some(data) }) => {
                    data.get("mode").and_then(|v| v.as_str()) == Some("ios")
                }
                _ => false,
            };

            if is_ios {
                // iOS mode: fetch element tree via WDA
                let wda_info = match &session_info {
                    Ok(DaemonResponse::Ok { data: Some(data) }) => data.clone(),
                    _ => {
                        output.error("No iOS session info");
                        return Err(anyhow::anyhow!("No iOS session info"));
                    }
                };

                let wda_port = wda_info
                    .get("wda_port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(8100) as u16;
                let wda_session_id = wda_info
                    .get("wda_session_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let scale = wda_info
                    .get("scale")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(3.0);

                let base_url = format!("http://localhost:{}", wda_port);
                let mut ref_map = RefMap::new();
                let tree = match platform::ios::fetch_element_tree(
                    &base_url,
                    wda_session_id,
                    scale,
                    &mut ref_map,
                    interactive,
                ) {
                    Ok(t) => t,
                    Err(e) => {
                        output.error(&format!("Failed to get iOS elements: {}", e));
                        return Err(anyhow::anyhow!("iOS elements failed: {}", e));
                    }
                };

                // Store refs
                let store_request = DaemonRequest::StoreRefs {
                    session: session_name.clone(),
                    refs: ref_map.refs.clone(),
                };
                let _ = client.request(store_request).await;

                let refs_json = json!(ref_map.refs);
                output.element_tree(&tree, &refs_json);
                return Ok(());
            }

            if is_android {
                // Android mode: use UIAutomator dump
                let adb = AdbClient::new(device.clone());

                let xml = match uiautomator::dump_ui(&adb) {
                    Ok(xml) => xml,
                    Err(e) => {
                        output.error(&format!("UIAutomator dump failed: {}", e));
                        return Err(anyhow::anyhow!("UIAutomator dump failed: {}", e));
                    }
                };

                let mut ref_map = RefMap::new();
                let tree = match uiautomator::parse_uiautomator_xml(&xml, &mut ref_map, interactive) {
                    Some(t) => t,
                    None => {
                        output.error("Failed to parse UIAutomator XML");
                        return Err(anyhow::anyhow!("Failed to parse UIAutomator XML"));
                    }
                };

                // Store refs in daemon
                let store_request = DaemonRequest::StoreRefs {
                    session: session_name.clone(),
                    refs: ref_map.refs.clone(),
                };
                let _ = client.request(store_request).await;

                let refs_json = json!(ref_map.refs);
                output.element_tree(&tree, &refs_json);
                return Ok(());
            }

            // Flutter mode: get semantics tree
            let sem_request = DaemonRequest::CallExtension {
                session: session_name.clone(),
                method: "ext.flutter.debugDumpSemanticsTreeInTraversalOrder".to_string(),
                args: None,
            };

            let sem_response = client.request(sem_request).await;
            let sem_data = match sem_response {
                Ok(DaemonResponse::Ok { data }) => data,
                Ok(DaemonResponse::Error { message }) => {
                    output.error(&message);
                    return Err(anyhow::anyhow!(message));
                }
                Ok(DaemonResponse::HasFlutterProcess { .. }) => {
                    output.error("Unexpected HasFlutterProcess response");
                    return Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"));
                }
                Err(e) => {
                    output.error(&e.to_string());
                    return Err(e.into());
                }
            };

            // Parse semantics tree
            let mut ref_map = RefMap::new();
            let mut tree = match sem_data
                .as_ref()
                .and_then(|d| parse_semantics_tree(d, &mut ref_map, interactive))
            {
                Some(t) => t,
                None => {
                    output.error("Failed to parse semantics tree");
                    return Err(anyhow::anyhow!("Failed to parse semantics tree"));
                }
            };

            // Extract scale factor from semantics text for style matching
            let scale_factor = sem_data
                .as_ref()
                .and_then(|d| d.get("data"))
                .and_then(|v| v.as_str())
                .and_then(|text| {
                    for line in text.lines() {
                        if line.contains("scaled by") {
                            if let Some(caps) = regex::Regex::new(r"scaled by ([\d.]+)x")
                                .ok()?
                                .captures(line)
                            {
                                return caps.get(1)?.as_str().parse().ok();
                            }
                        }
                    }
                    None
                })
                .unwrap_or(1.0);

            // Also fetch render tree and merge styles
            let render_request = DaemonRequest::CallExtension {
                session: session_name.clone(),
                method: "ext.flutter.debugDumpRenderTree".to_string(),
                args: None,
            };

            if let Ok(DaemonResponse::Ok {
                data: Some(render_data),
            }) = client.request(render_request).await
            {
                if let Some(render_text) = render_data.get("data").and_then(|v| v.as_str()) {
                    // Parse render tree for styles
                    let render_styles = parse_render_tree(render_text, scale_factor);
                    // Match styles to semantic nodes
                    match_styles_to_nodes(&mut tree, &render_styles, scale_factor);
                }
            }

            // Store refs in daemon
            let store_request = DaemonRequest::StoreRefs {
                session: session_name.clone(),
                refs: ref_map.refs.clone(),
            };
            let _ = client.request(store_request).await;

            let refs_json = json!(ref_map.refs);
            output.element_tree(&tree, &refs_json);
            Ok(())
        }

        // Tap command
        Commands::Tap { reference } => {
            tap_action(&mut client, &output, &session_name, &reference, args.json).await
        }

        // DoubleTap command
        Commands::DoubleTap { reference } => {
            double_tap_action(&mut client, &output, &session_name, &reference, args.json).await
        }

        // LongPress command
        Commands::LongPress {
            reference,
            duration,
        } => {
            long_press_action(
                &mut client,
                &output,
                &session_name,
                &reference,
                duration,
                args.json,
            )
            .await
        }

        // Clear command
        Commands::Clear { reference } => {
            clear_action(&mut client, &output, &session_name, &reference, args.json).await
        }

        // Text command (input text into element)
        Commands::Text {
            reference,
            text,
            clear,
        } => {
            input_action(
                &mut client,
                &output,
                &session_name,
                &reference,
                &text,
                clear,
                args.json,
            )
            .await
        }

        // Screenshot command
        Commands::Screenshot { path } => {
            capture_action(&mut client, &output, &session_name, device.clone(), path, args.json).await
        }

        // Scroll command
        Commands::Scroll {
            direction,
            distance,
        } => {
            scroll_action(
                &mut client,
                &output,
                &session_name,
                &direction,
                distance,
                args.json,
            )
            .await
        }

        // Swipe command
        Commands::Swipe { direction, from } => {
            swipe_action(
                &mut client,
                &output,
                &session_name,
                &direction,
                from,
                args.json,
            )
            .await
        }

        // Get command - get element property
        Commands::Get {
            property,
            reference,
        } => {
            get_action(
                &mut client,
                &output,
                &session_name,
                &property,
                &reference,
                args.json,
            )
            .await
        }

        // Is command - check element state
        Commands::Is { state, reference } => {
            is_action(
                &mut client,
                &output,
                &session_name,
                &state,
                &reference,
                args.json,
            )
            .await
        }

        // Flutter subcommands
        Commands::Flutter(flutter_cmd) => match flutter_cmd {
            FlutterCommands::Reload => {
                // Check if session has flutter process (run mode)
                let has_process_req = DaemonRequest::HasFlutterProcess {
                    session: session_name.clone(),
                };

                let has_process = matches!(
                    client.request(has_process_req).await,
                    Ok(DaemonResponse::HasFlutterProcess { has_process: true })
                );

                if has_process {
                    // Running via mobile-use run - tell user to use that terminal
                    output.info("Hot reload: Press 'r' in the mobile-use run terminal.\n\
                                (The flutter process stdin is managed by the run command)");
                    Ok(())
                } else {
                    // VM Service mode - use existing implementation
                    let request = DaemonRequest::CallExtension {
                        session: session_name,
                        method: "ext.flutter.reassemble".to_string(),
                        args: None,
                    };
                    handle_response!(output, client.request(request).await, |_data: Option<serde_json::Value>| {
                        output.action_result(&ActionResult {
                            success: true,
                            message: Some("Hot reload successful".to_string()),
                            data: None,
                        });
                        Ok(())
                    })
                }
            }

            FlutterCommands::Restart => {
                // Check if session has flutter process (run mode)
                let has_process_req = DaemonRequest::HasFlutterProcess {
                    session: session_name.clone(),
                };

                let has_process = matches!(
                    client.request(has_process_req).await,
                    Ok(DaemonResponse::HasFlutterProcess { has_process: true })
                );

                if has_process {
                    // Running via mobile-use run - tell user to use that terminal
                    output.info("Hot restart: Press 'R' in the mobile-use run terminal.\n\
                                (The flutter process stdin is managed by the run command)");
                    Ok(())
                } else {
                    // No flutter process - show error with solutions
                    output.error("Hot restart not available in this connection mode.\n\n\
                        Reason: Connected via VM Service URL only. Hot restart requires\n\
                        control of the flutter process stdin.\n\n\
                        Solutions:\n\
                        1. Use `mobile-use run` to launch the app (recommended)\n\
                           - Then press 'R' in that terminal for hot restart\n\
                        2. If flutter run is in another terminal, press 'R' there manually\n\
                        3. Use `flutter reload` instead (works for most code changes)");
                    Err(anyhow::anyhow!("Hot restart not available - use mobile-use run"))
                }
            }

            FlutterCommands::Widgets => {
                let request = DaemonRequest::CallExtension {
                    session: session_name,
                    method: "ext.flutter.debugDumpRenderTree".to_string(),
                    args: None,
                };
                handle_response!(output, client.request(request).await, |data: Option<
                    serde_json::Value,
                >| {
                    if let Some(data) = data {
                        output.raw(&serde_json::to_string_pretty(&data).unwrap_or_default());
                    }
                    Ok(())
                })
            }
        }

        // Wait command
        Commands::Wait {
            target,
            text,
            timeout: timeout_ms,
        } => {
            // Determine what we're waiting for
            if let Some(text_value) = text {
                // Wait for text - search through all elements for matching label
                let poll_interval = Duration::from_millis(500);
                let timeout_duration = Duration::from_millis(timeout_ms as u64);

                let result = tokio::time::timeout(timeout_duration, async {
                    loop {
                        // Get current elements
                        let sem_request = DaemonRequest::CallExtension {
                            session: session_name.clone(),
                            method: "ext.flutter.debugDumpSemanticsTreeInTraversalOrder"
                                .to_string(),
                            args: None,
                        };

                        if let Ok(DaemonResponse::Ok {
                            data: Some(sem_data),
                        }) = client.request(sem_request).await
                        {
                            // Check if text appears anywhere in the semantics dump
                            if let Some(text_content) =
                                sem_data.get("data").and_then(|v| v.as_str())
                            {
                                if text_content.contains(&text_value) {
                                    return Ok(());
                                }
                            }
                        }

                        tokio::time::sleep(poll_interval).await;
                    }
                })
                .await;

                match result {
                    Ok(Ok(())) => {
                        output.action_result(&ActionResult {
                            success: true,
                            message: Some(format!("Text \"{}\" appeared", text_value)),
                            data: None,
                        });
                        Ok(())
                    }
                    Ok(Err(e)) => Err(e),
                    Err(_) => {
                        output.error(&format!(
                            "Timeout: text \"{}\" did not appear within {}ms",
                            text_value, timeout_ms
                        ));
                        Err(anyhow::anyhow!("Timeout waiting for text"))
                    }
                }
            } else if let Some(target_value) = target {
                // Try to parse as duration (number)
                if let Ok(duration) = target_value.parse::<u64>() {
                    // Wait for specified duration
                    tokio::time::sleep(Duration::from_millis(duration)).await;
                    output.action_result(&ActionResult {
                        success: true,
                        message: Some(format!("Waited for {}ms", duration)),
                        data: None,
                    });
                    Ok(())
                } else {
                    // Treat as element reference - wait for element to appear
                    let normalized_ref = target_value.trim_start_matches('@');
                    let poll_interval = Duration::from_millis(500);
                    let timeout_duration = Duration::from_millis(timeout_ms as u64);

                    let result = tokio::time::timeout(timeout_duration, async {
                        loop {
                            // Get current elements and check if ref exists
                            let sem_request = DaemonRequest::CallExtension {
                                session: session_name.clone(),
                                method: "ext.flutter.debugDumpSemanticsTreeInTraversalOrder"
                                    .to_string(),
                                args: None,
                            };

                            if let Ok(DaemonResponse::Ok {
                                data: Some(sem_data),
                            }) = client.request(sem_request).await
                            {
                                let mut ref_map = RefMap::new();
                                if parse_semantics_tree(&sem_data, &mut ref_map, false).is_some() {
                                    if ref_map.get(normalized_ref).is_some() {
                                        return Ok(());
                                    }
                                }
                            }

                            tokio::time::sleep(poll_interval).await;
                        }
                    })
                    .await;

                    match result {
                        Ok(Ok(())) => {
                            output.action_result(&ActionResult {
                                success: true,
                                message: Some(format!("Element @{} appeared", normalized_ref)),
                                data: None,
                            });
                            Ok(())
                        }
                        Ok(Err(e)) => Err(e),
                        Err(_) => {
                            output.error(&format!(
                                "Timeout: element @{} did not appear within {}ms",
                                normalized_ref, timeout_ms
                            ));
                            Err(anyhow::anyhow!("Timeout waiting for element"))
                        }
                    }
                }
            } else {
                output
                    .error("Wait requires a target (duration in ms, element reference, or --text)");
                Err(anyhow::anyhow!("Wait requires a target"))
            }
        }

        // ScrollTo command - not yet implemented
        Commands::ScrollTo { reference: _ } => {
            output.error("ScrollTo command not yet implemented for daemon mode");
            Err(anyhow::anyhow!(
                "ScrollTo command not yet implemented for daemon mode"
            ))
        }

        // Run command - spawn flutter run and auto-connect
        Commands::Run { apk, package, args: flutter_args } => {
            // Detect project type
            let project_type = detect_project_type(&apk, &package);

            match project_type {
                ProjectType::FlutterAndroid => {
                    // 1. Spawn flutter run --machine with provided args
                    let (manager, mut event_rx) = match FlutterProcessManager::spawn(flutter_args).await {
                        Ok(result) => result,
                        Err(e) => {
                            output.error(&format!("Failed to spawn flutter run: {}", e));
                            return Err(e);
                        }
                    };

                    // 2. Write PID file so 'quit' command can find us
                    let pid_path = get_run_pid_path(&session_name);
                    let _ = tokio::fs::write(&pid_path, std::process::id().to_string()).await;

                    // 3. Setup stdin forwarding task
                    let stdin_tx = manager.get_stdin_sender();
                    let stdin_task = tokio::spawn(async move {
                        let stdin = tokio::io::stdin();
                        let mut reader = BufReader::new(stdin);
                        let mut line = String::new();
                        loop {
                            line.clear();
                            match reader.read_line(&mut line).await {
                                Ok(0) => break, // EOF
                                Ok(_) => {
                                    let input = line.trim();
                                    if !input.is_empty() {
                                        let _ = stdin_tx.send(input.to_string()).await;
                                    }
                                }
                                Err(_) => break,
                            }
                        }
                    });

                    // 4. Setup SIGTERM handler for graceful shutdown via 'quit' command
                    let mut sigterm = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate(),
                    )?;

                    // 5. Process event loop with signal handling
                    let mut connected = false;
                    let mut stop_requested = false;
                    loop {
                        tokio::select! {
                            event = event_rx.recv() => {
                                match event {
                                    Some(FlutterEvent::Log(line)) => {
                                        println!("{}", line);
                                    }
                                    Some(FlutterEvent::AppDebugPort(info)) => {
                                        println!("Flutter app debug port: {}", info.ws_uri);
                                        // Auto-connect to daemon
                                        if !connected {
                                            let connect_request = DaemonRequest::Connect {
                                                session: session_name.clone(),
                                                device: device.clone(),
                                                url: Some(info.ws_uri.clone()),
                                                port: None,
                                            };
                                            match client.request(connect_request).await {
                                                Ok(DaemonResponse::Ok { .. }) => {
                                                    output.success(&format!("Auto-connected to {}", info.ws_uri));
                                                    // Register flutter process with daemon
                                                    let register_request = DaemonRequest::RegisterFlutterProcess {
                                                        session: session_name.clone(),
                                                    };
                                                    let _ = client.request(register_request).await;
                                                    connected = true;
                                                }
                                                Ok(resp) => {
                                                    output.error(&format!("Unexpected response: {:?}", resp));
                                                }
                                                Err(e) => {
                                                    output.error(&format!("Failed to auto-connect: {}", e));
                                                }
                                            }
                                        }
                                    }
                                    Some(FlutterEvent::AppStarted(app_id)) => {
                                        output.success(&format!("App started: {}", app_id));
                                    }
                                    Some(FlutterEvent::AppStopped(app_id)) => {
                                        output.info(&format!("App stopped: {}", app_id));
                                        break;
                                    }
                                    Some(FlutterEvent::Error(e)) => {
                                        output.error(&e);
                                    }
                                    None => break, // Channel closed
                                }
                            }
                            _ = sigterm.recv() => {
                                if !stop_requested {
                                    output.info("Received stop signal, shutting down...");
                                    let _ = manager.send_input("q").await;
                                    stop_requested = true;
                                }
                            }
                        }
                    }

                    // 6. Cleanup
                    stdin_task.abort();
                    let _ = tokio::fs::remove_file(&pid_path).await;
                    if connected {
                        let disconnect_req = DaemonRequest::Disconnect {
                            session: session_name.clone(),
                        };
                        let _ = client.request(disconnect_req).await;
                    }
                    Ok(())
                }

                ProjectType::NativeAndroid => {
                    let project_dir = std::env::current_dir().unwrap_or_default();
                    let adb = AdbClient::new(device.clone());

                    // Determine package name and optionally build/install
                    let final_package = if let Some(ref apk_path) = apk {
                        // APK provided: install it
                        output.info(&format!("Installing APK: {}", apk_path));
                        if let Err(e) = gradle::install_apk(&adb, std::path::Path::new(apk_path)) {
                            output.error(&format!("Failed to install APK: {}", e));
                            return Err(anyhow::anyhow!("Failed to install APK: {}", e));
                        }
                        // Package must be provided with APK
                        match &package {
                            Some(pkg) => pkg.clone(),
                            None => {
                                output.error("--package is required when using an APK file");
                                return Err(anyhow::anyhow!("--package is required when using an APK file"));
                            }
                        }
                    } else if let Some(ref pkg) = package {
                        // Package provided directly: just launch it
                        pkg.clone()
                    } else {
                        // Detect from Gradle project
                        let modules = match gradle::find_gradle_modules(&project_dir) {
                            Ok(m) if !m.is_empty() => m,
                            Ok(_) => {
                                output.error("No Android application modules found in this project");
                                return Err(anyhow::anyhow!("No Android application modules found"));
                            }
                            Err(e) => {
                                output.error(&format!("Failed to detect Gradle modules: {}", e));
                                return Err(anyhow::anyhow!("Failed to detect Gradle modules: {}", e));
                            }
                        };

                        // Select module
                        let module = if modules.len() == 1 {
                            &modules[0]
                        } else if output.is_human() {
                            // Interactive selection
                            output.info("Multiple app modules found:");
                            for (i, m) in modules.iter().enumerate() {
                                let pkg_info = m.package.as_deref().unwrap_or("unknown");
                                output.info(&format!("  {}. {} ({})", i + 1, m.name, pkg_info));
                            }
                            eprint!("\nSelect module [1-{}]: ", modules.len());
                            let mut input = String::new();
                            std::io::stdin().read_line(&mut input)?;
                            let choice: usize = input.trim().parse().unwrap_or(0);
                            if choice < 1 || choice > modules.len() {
                                output.error(&format!("Invalid selection '{}'. Use 1-{}.", input.trim(), modules.len()));
                                return Err(anyhow::anyhow!("Invalid module selection"));
                            }
                            &modules[choice - 1]
                        } else {
                            // JSON mode: auto-select first
                            &modules[0]
                        };

                        let pkg = match &module.package {
                            Some(p) => p.clone(),
                            None => {
                                output.error(&format!(
                                    "Could not detect applicationId for module '{}'. Use --package.",
                                    module.name
                                ));
                                return Err(anyhow::anyhow!("Could not detect applicationId"));
                            }
                        };

                        // Build APK
                        output.info(&format!("Building {}...", module.name));
                        let apk_path = match gradle::build_apk(&project_dir, module) {
                            Ok(p) => p,
                            Err(e) => {
                                output.error(&format!("Build failed: {}", e));
                                return Err(anyhow::anyhow!("Build failed: {}", e));
                            }
                        };

                        // Install APK
                        output.info("Installing APK...");
                        if let Err(e) = gradle::install_apk(&adb, &apk_path) {
                            output.error(&format!("Install failed: {}", e));
                            return Err(anyhow::anyhow!("Install failed: {}", e));
                        }

                        pkg
                    };

                    // Launch the app
                    output.info(&format!("Launching {}...", final_package));
                    if let Err(e) = gradle::launch_app(&adb, &final_package) {
                        output.error(&format!("Launch failed: {}", e));
                        return Err(anyhow::anyhow!("Launch failed: {}", e));
                    }

                    // Wait for app to start
                    tokio::time::sleep(Duration::from_secs(2)).await;

                    // Create Android session in daemon
                    let connect_request = DaemonRequest::ConnectAndroid {
                        session: session_name.clone(),
                        device: device.clone(),
                        package: final_package.clone(),
                    };
                    match client.request(connect_request).await {
                        Ok(DaemonResponse::Ok { .. }) => {
                            output.success(&format!("Connected to Android app: {}", final_package));
                        }
                        Ok(resp) => {
                            output.error(&format!("Unexpected response: {:?}", resp));
                        }
                        Err(e) => {
                            output.error(&format!("Failed to create session: {}", e));
                        }
                    }

                    // Write PID file so 'quit' command can find us
                    let pid_path = get_run_pid_path(&session_name);
                    let _ = tokio::fs::write(&pid_path, std::process::id().to_string()).await;

                    // Setup SIGTERM handler for graceful shutdown
                    let mut sigterm = tokio::signal::unix::signal(
                        tokio::signal::unix::SignalKind::terminate(),
                    )?;

                    output.info("Android app running. Press Ctrl+C or use 'mobile-use quit' to stop.");

                    // Wait for signal
                    tokio::select! {
                        _ = sigterm.recv() => {
                            output.info("Received stop signal, shutting down...");
                        }
                        _ = tokio::signal::ctrl_c() => {
                            output.info("Received Ctrl+C, shutting down...");
                        }
                    }

                    // Cleanup: force-stop the app
                    let _ = adb.shell(&format!("am force-stop {}", final_package));
                    output.info(&format!("Stopped {}", final_package));

                    // Remove PID file
                    let _ = tokio::fs::remove_file(&pid_path).await;

                    // Disconnect session
                    let disconnect_req = DaemonRequest::Disconnect {
                        session: session_name.clone(),
                    };
                    let _ = client.request(disconnect_req).await;

                    Ok(())
                }

                ProjectType::Unknown => {
                    output.error("Cannot detect project type. Run this command from a Flutter or Android project directory, or specify --apk/--package for native Android.");
                    Err(anyhow::anyhow!("Cannot detect project type"))
                }
            }
        }
    };

    if result.is_err() {
        std::process::exit(1);
    }

    Ok(())
}

// Helper function to execute device actions via daemon's DeviceOperator
async fn execute_device_action(
    client: &mut DaemonClient,
    session: &str,
    action: DeviceAction,
) -> Result<Option<serde_json::Value>> {
    let request = DaemonRequest::ExecuteAction {
        session: session.to_string(),
        action,
    };
    match client.request(request).await? {
        DaemonResponse::Ok { data } => Ok(data),
        DaemonResponse::Error { message } => Err(anyhow::anyhow!(message)),
        _ => Err(anyhow::anyhow!("Unexpected response")),
    }
}

// Helper function to clear text from a text field via daemon
async fn clear_text_field_via_daemon(
    client: &mut DaemonClient,
    session: &str,
) -> Result<()> {
    // Move to end
    execute_device_action(
        client,
        session,
        DeviceAction::Keyevent {
            key: "MOVE_END".to_string(),
        },
    )
    .await?;
    // Delete backwards
    for _ in 0..MAX_CLEAR_DELETE_PRESSES {
        execute_device_action(
            client,
            session,
            DeviceAction::Keyevent {
                key: "DEL".to_string(),
            },
        )
        .await?;
    }
    Ok(())
}

// Helper function to resolve element reference to bounds
async fn resolve_ref(
    client: &mut DaemonClient,
    session: &str,
    reference: &str,
) -> Result<ElementRef> {
    let resolve_req = DaemonRequest::ResolveRef {
        session: session.to_string(),
        reference: reference.to_string(),
    };
    match client.request(resolve_req).await? {
        DaemonResponse::Ok { data: Some(data) } => {
            let element: ElementRef = serde_json::from_value(data)?;
            Ok(element)
        }
        DaemonResponse::Ok { data: None } => {
            Err(anyhow::anyhow!("Element not found: {}", reference))
        }
        DaemonResponse::Error { message } => Err(anyhow::anyhow!(message)),
        DaemonResponse::HasFlutterProcess { .. } => {
            Err(anyhow::anyhow!("Unexpected HasFlutterProcess response"))
        }
    }
}

// Tap action implementation
async fn tap_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let (x, y) = element.bounds.center();
    execute_device_action(
        client,
        session,
        DeviceAction::Tap {
            x: x as i32,
            y: y as i32,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Tapped: {} \"{}\"",
            element.element_type,
            element.label.as_deref().unwrap_or("")
        )),
        data: None,
    });
    Ok(())
}

// DoubleTap action implementation
async fn double_tap_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let (x, y) = element.bounds.center();
    execute_device_action(
        client,
        session,
        DeviceAction::DoubleTap {
            x: x as i32,
            y: y as i32,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Double-tapped: {} \"{}\"",
            element.element_type,
            element.label.as_deref().unwrap_or("")
        )),
        data: None,
    });
    Ok(())
}

// LongPress action implementation
async fn long_press_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    duration: u32,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let (x, y) = element.bounds.center();
    execute_device_action(
        client,
        session,
        DeviceAction::LongPress {
            x: x as i32,
            y: y as i32,
            duration_ms: duration,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Long-pressed: {} for {}ms",
            element.label.as_deref().unwrap_or(reference),
            duration
        )),
        data: None,
    });
    Ok(())
}

// Clear action implementation
async fn clear_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    // Tap to focus
    let (x, y) = element.bounds.center();
    execute_device_action(
        client,
        session,
        DeviceAction::Tap {
            x: x as i32,
            y: y as i32,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Move to end and delete backwards
    clear_text_field_via_daemon(client, session).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Cleared: {}",
            element.label.as_deref().unwrap_or(reference)
        )),
        data: None,
    });
    Ok(())
}

// Input action implementation
async fn input_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    reference: &str,
    text: &str,
    clear_first: bool,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    // Tap to focus
    let (x, y) = element.bounds.center();
    execute_device_action(
        client,
        session,
        DeviceAction::Tap {
            x: x as i32,
            y: y as i32,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Clear if requested
    if clear_first {
        clear_text_field_via_daemon(client, session).await.map_err(|e| {
            output.error(&e.to_string());
            e
        })?;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Input text
    execute_device_action(
        client,
        session,
        DeviceAction::InputText {
            text: text.to_string(),
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Input: \"{}\" -> {}",
            text,
            element.label.as_deref().unwrap_or(reference)
        )),
        data: None,
    });
    Ok(())
}

// Capture action implementation
async fn capture_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    _device: Option<String>,
    path: Option<String>,
    _json_mode: bool,
) -> Result<()> {
    // Determine output path
    let output_path = path.unwrap_or_else(|| {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        format!("screenshot-{}.png", timestamp)
    });

    // Take screenshot via daemon's DeviceOperator
    execute_device_action(
        client,
        session,
        DeviceAction::Screenshot {
            path: output_path.clone(),
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    // Get image dimensions if possible
    let dimensions = if let Ok(img) = image::open(&output_path) {
        format!("{}x{}", img.width(), img.height())
    } else {
        "unknown".to_string()
    };

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!(
            "Screenshot saved: {} ({})",
            output_path, dimensions
        )),
        data: None,
    });
    Ok(())
}

// Scroll action implementation
async fn scroll_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    direction: &str,
    distance: i32,
    _json_mode: bool,
) -> Result<()> {
    let dir: Direction = direction.parse().map_err(|e: String| {
        output.error(&e);
        anyhow::anyhow!(e)
    })?;

    // Get screen size via daemon
    let size_data = execute_device_action(client, session, DeviceAction::GetScreenSize)
        .await
        .map_err(|e| {
            output.error(&e.to_string());
            e
        })?;
    let (width, height) = match size_data {
        Some(data) => {
            let w = data["width"].as_i64().unwrap_or(1080) as i32;
            let h = data["height"].as_i64().unwrap_or(1920) as i32;
            (w, h)
        }
        None => (1080, 1920),
    };
    let cx = width / 2;
    let cy = height / 2;

    let (x1, y1, x2, y2) = match dir {
        Direction::Up => (cx, cy + distance / 2, cx, (cy - distance / 2).max(0)),
        Direction::Down => (cx, (cy - distance / 2).max(0), cx, cy + distance / 2),
        Direction::Left => (cx + distance / 2, cy, (cx - distance / 2).max(0), cy),
        Direction::Right => ((cx - distance / 2).max(0), cy, cx + distance / 2, cy),
    };

    execute_device_action(
        client,
        session,
        DeviceAction::Swipe {
            x1,
            y1,
            x2,
            y2,
            duration_ms: 300,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!("Scrolled {} by {}", direction, distance)),
        data: None,
    });
    Ok(())
}

// Swipe action implementation
async fn swipe_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    direction: &str,
    from: Option<String>,
    _json_mode: bool,
) -> Result<()> {
    let dir: Direction = direction.parse().map_err(|e: String| {
        output.error(&e);
        anyhow::anyhow!(e)
    })?;

    // Get start position
    let (start_x, start_y) = if let Some(ref ref_id) = from {
        let element = resolve_ref(client, session, ref_id).await.map_err(|e| {
            output.error(&e.to_string());
            e
        })?;
        let (x, y) = element.bounds.center();
        (x as i32, y as i32)
    } else {
        let size_data = execute_device_action(client, session, DeviceAction::GetScreenSize)
            .await
            .map_err(|e| {
                output.error(&e.to_string());
                e
            })?;
        match size_data {
            Some(data) => {
                let w = data["width"].as_i64().unwrap_or(1080) as i32;
                let h = data["height"].as_i64().unwrap_or(1920) as i32;
                (w / 2, h / 2)
            }
            None => (540, 960),
        }
    };

    let (end_x, end_y) = match dir {
        Direction::Up => (start_x, (start_y - DEFAULT_SWIPE_DISTANCE).max(0)),
        Direction::Down => (start_x, start_y + DEFAULT_SWIPE_DISTANCE),
        Direction::Left => ((start_x - DEFAULT_SWIPE_DISTANCE).max(0), start_y),
        Direction::Right => (start_x + DEFAULT_SWIPE_DISTANCE, start_y),
    };

    execute_device_action(
        client,
        session,
        DeviceAction::Swipe {
            x1: start_x,
            y1: start_y,
            x2: end_x,
            y2: end_y,
            duration_ms: 200,
        },
    )
    .await
    .map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    output.action_result(&ActionResult {
        success: true,
        message: Some(format!("Swiped {}", direction)),
        data: None,
    });
    Ok(())
}

// Get action implementation
async fn get_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    property: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let value = match property.to_lowercase().as_str() {
        "text" => element.label.clone().unwrap_or_default(),
        "prop" | "property" => {
            serde_json::to_string(&element.properties).unwrap_or_else(|_| "{}".to_string())
        }
        "bounds" => format!(
            "{},{},{},{}",
            element.bounds.x, element.bounds.y, element.bounds.width, element.bounds.height
        ),
        "type" => element.element_type.clone(),
        _ => {
            // Look up in properties
            element
                .properties
                .get(property)
                .map(|v| {
                    if v.is_string() {
                        v.as_str().unwrap_or("").to_string()
                    } else {
                        v.to_string()
                    }
                })
                .unwrap_or_else(|| format!("Unknown property: {}", property))
        }
    };

    output.action_result(&ActionResult {
        success: true,
        message: Some(value.clone()),
        data: Some(json!({
            "property": property,
            "value": value,
            "ref": reference
        })),
    });
    Ok(())
}

// Is action implementation
async fn is_action(
    client: &mut DaemonClient,
    output: &OutputFormatter,
    session: &str,
    state: &str,
    reference: &str,
    _json_mode: bool,
) -> Result<()> {
    let element = resolve_ref(client, session, reference).await.map_err(|e| {
        output.error(&e.to_string());
        e
    })?;

    let result = match state.to_lowercase().as_str() {
        "visible" => true, // If element exists and was found, it's visible
        "enabled" => !element
            .properties
            .get("isDisabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "checked" => element
            .properties
            .get("isChecked")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        "focused" => element
            .properties
            .get("isFocused")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
        _ => {
            output.error(&format!(
                "Unknown state: {}. Valid states: visible, enabled, checked, focused",
                state
            ));
            return Err(anyhow::anyhow!(
                "Unknown state: {}. Valid states: visible, enabled, checked, focused",
                state
            ));
        }
    };

    output.action_result(&ActionResult {
        success: true,
        message: Some(result.to_string()),
        data: Some(json!({
            "state": state,
            "value": result,
            "ref": reference
        })),
    });
    Ok(())
}
