// This module is from pre-daemon architecture and is currently unused.
// Kept for potential future use or reference.
#![allow(dead_code)]

use crate::core::{MobileUseError, AppInfo, ElementRef, RefMap, Result};
use crate::platform::android::AdbClient;
use crate::platform::flutter::VmServiceClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;

/// Persistent session state (saved to file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub platform: String,
    pub device: Option<String>,
    pub vm_service_url: Option<String>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            platform: "flutter".to_string(),
            device: None,
            vm_service_url: None,
        }
    }
}

/// Session state for the current execution
pub struct Session {
    pub name: String,
    pub state: SessionState,
    pub vm_service: VmServiceClient,
    pub adb: AdbClient,
    pub ref_map: RefMap,
    pub connected: bool,
    session_dir: PathBuf,
}

impl Session {
    /// Create a new session (loads state from file if exists)
    pub fn load(name: &str, platform: Option<&str>, device: Option<String>) -> Result<Self> {
        let session_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("mobile-use")
            .join("sessions");

        std::fs::create_dir_all(&session_dir)?;

        // Try to load existing session state
        let state_file = session_dir.join(format!("{}.json", name));
        let mut state = if state_file.exists() {
            let json = std::fs::read_to_string(&state_file)?;
            serde_json::from_str(&json).unwrap_or_default()
        } else {
            SessionState::default()
        };

        // Override with CLI arguments if provided
        if let Some(p) = platform {
            if p != "flutter" {
                state.platform = p.to_string();
            }
        }
        if device.is_some() {
            state.device = device;
        }

        // Load ref_map if exists
        let ref_file = session_dir.join(format!("{}.refs.json", name));
        let ref_map = if ref_file.exists() {
            let json = std::fs::read_to_string(&ref_file)?;
            let refs: HashMap<String, ElementRef> = serde_json::from_str(&json).unwrap_or_default();
            let counter = refs
                .keys()
                .filter_map(|k| k.strip_prefix('e').and_then(|n| n.parse::<u32>().ok()))
                .max()
                .unwrap_or(0);
            RefMap::with_refs(refs, counter)
        } else {
            RefMap::new()
        };

        let adb = AdbClient::new(state.device.clone());

        Ok(Self {
            name: name.to_string(),
            state,
            vm_service: VmServiceClient::new(),
            adb,
            ref_map,
            connected: false,
            session_dir,
        })
    }

    /// Auto-reconnect to VM Service if we have a saved URL
    pub async fn auto_reconnect(&mut self) -> Result<bool> {
        if let Some(ref url) = self.state.vm_service_url {
            info!("Auto-reconnecting to {}", url);
            match self.vm_service.connect(url).await {
                Ok(_) => {
                    self.connected = true;
                    info!("Auto-reconnect successful");
                    Ok(true)
                }
                Err(e) => {
                    info!("Auto-reconnect failed: {}, clearing saved URL", e);
                    self.state.vm_service_url = None;
                    self.save_state()?;
                    Ok(false)
                }
            }
        } else {
            Ok(false)
        }
    }

    /// Save session state to file
    pub fn save_state(&self) -> Result<()> {
        let state_file = self.session_dir.join(format!("{}.json", self.name));
        let json = serde_json::to_string_pretty(&self.state)?;
        std::fs::write(&state_file, json)?;
        info!("Saved session state to {:?}", state_file);
        Ok(())
    }

    /// Save ref_map to file
    pub fn save_refs(&self) -> Result<()> {
        let ref_file = self.session_dir.join(format!("{}.refs.json", self.name));
        let json = serde_json::to_string_pretty(&self.ref_map.refs)?;
        std::fs::write(&ref_file, json)?;
        info!("Saved {} refs to {:?}", self.ref_map.refs.len(), ref_file);
        Ok(())
    }

    /// Set VM Service URL and save
    pub fn set_vm_url(&mut self, url: &str) -> Result<()> {
        self.state.vm_service_url = Some(url.to_string());
        self.save_state()
    }

    /// Clear session (disconnect)
    pub fn clear(&mut self) -> Result<()> {
        self.state.vm_service_url = None;
        self.connected = false;
        self.ref_map.clear();
        self.save_state()?;

        // Delete refs file
        let ref_file = self.session_dir.join(format!("{}.refs.json", self.name));
        let _ = std::fs::remove_file(ref_file);

        Ok(())
    }

    /// Get app info for display
    pub fn app_info(&self) -> AppInfo {
        AppInfo {
            platform: self.state.platform.clone(),
            device: self.state.device.clone(),
            app_id: None,
            vm_service_url: self.state.vm_service_url.clone(),
            connected: self.connected,
        }
    }

    /// Ensure connected, return error if not
    pub fn ensure_connected(&self) -> Result<()> {
        if !self.connected {
            Err(MobileUseError::NotConnected)
        } else {
            Ok(())
        }
    }
}
