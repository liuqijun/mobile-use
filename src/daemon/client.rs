use crate::core::{MobileUseError, Result};
use crate::daemon::{get_pid_path, get_socket_path, DaemonRequest, DaemonResponse};
use std::process::Command;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// Maximum number of retries when connecting to the daemon
const DAEMON_CONNECT_MAX_RETRIES: u32 = 50;
/// Interval between connection retry attempts in milliseconds
const DAEMON_CONNECT_RETRY_INTERVAL_MS: u64 = 100;

/// Client for communicating with the daemon
pub struct DaemonClient {
    /// Connection to daemon
    stream: BufReader<UnixStream>,
}

impl DaemonClient {
    /// Connect to the daemon, starting it if necessary
    pub async fn connect() -> Result<Self> {
        let socket_path = get_socket_path();

        // Try to connect directly first
        match UnixStream::connect(&socket_path).await {
            Ok(stream) => {
                debug!("Connected to daemon at {:?}", socket_path);
                return Ok(Self {
                    stream: BufReader::new(stream),
                });
            }
            Err(e) => {
                debug!("Failed to connect to daemon: {}, attempting to start", e);
            }
        }

        // Start daemon and retry
        Self::start_daemon()?;

        // Retry connection up to DAEMON_CONNECT_MAX_RETRIES times
        let mut last_error = String::new();
        for attempt in 1..=DAEMON_CONNECT_MAX_RETRIES {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                DAEMON_CONNECT_RETRY_INTERVAL_MS,
            ))
            .await;

            match UnixStream::connect(&socket_path).await {
                Ok(stream) => {
                    info!(
                        "Connected to daemon at {:?} after {} attempts",
                        socket_path, attempt
                    );
                    return Ok(Self {
                        stream: BufReader::new(stream),
                    });
                }
                Err(e) => {
                    last_error = e.to_string();
                    debug!("Connection attempt {} failed: {}", attempt, e);
                }
            }
        }

        Err(MobileUseError::ConnectionFailed(format!(
            "Failed to connect to daemon after {} attempts: {}",
            DAEMON_CONNECT_MAX_RETRIES, last_error
        )))
    }

    /// Start the daemon process
    fn start_daemon() -> Result<()> {
        let exe_path = std::env::current_exe().map_err(|e| {
            MobileUseError::Other(format!("Failed to get current executable path: {}", e))
        })?;

        info!("Starting daemon process: {:?} daemon start", exe_path);

        Command::new(&exe_path)
            .args(["daemon", "start"])
            .spawn()
            .map_err(|e| MobileUseError::Other(format!("Failed to spawn daemon process: {}", e)))?;

        Ok(())
    }

    /// Check if the daemon is running by checking PID file and process
    pub fn is_daemon_running() -> bool {
        let pid_path = get_pid_path();

        // Check if PID file exists
        if !pid_path.exists() {
            return false;
        }

        // Read PID from file
        let pid_str = match std::fs::read_to_string(&pid_path) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let pid: i32 = match pid_str.trim().parse() {
            Ok(p) => p,
            Err(_) => return false,
        };

        // Use kill -0 to check if process is alive
        // On Unix, this sends signal 0 which doesn't kill the process but checks if it exists
        #[cfg(unix)]
        {
            Command::new("kill")
                .args(["-0", &pid.to_string()])
                .output()
                .map(|output| output.status.success())
                .unwrap_or(false)
        }

        #[cfg(not(unix))]
        {
            // On non-Unix systems, just check if PID file exists
            true
        }
    }

    /// Send a request to the daemon and receive a response
    pub async fn request(&mut self, request: DaemonRequest) -> Result<DaemonResponse> {
        // Serialize request to JSON + newline
        let request_json = serde_json::to_string(&request)?;
        debug!("Sending request: {}", request_json);

        // Write to stream
        self.stream
            .get_mut()
            .write_all(request_json.as_bytes())
            .await?;
        self.stream.get_mut().write_all(b"\n").await?;
        self.stream.get_mut().flush().await?;

        // Read response line
        let mut response_line = String::new();
        self.stream.read_line(&mut response_line).await?;

        debug!("Received response: {}", response_line.trim());

        // Parse as DaemonResponse
        let response: DaemonResponse = serde_json::from_str(response_line.trim())?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_start_daemon_gets_exe_path() {
        // Verify current_exe works
        let exe_path = std::env::current_exe();
        assert!(exe_path.is_ok());
    }
}
