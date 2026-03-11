use crate::core::types::{DeviceOperator, Platform};
use crate::core::RefMap;
use crate::platform::android::AdbClient;
use crate::platform::flutter::VmServiceClient;
use std::collections::HashMap;
use tracing::info;

/// A single daemon session managing a connection to a Flutter app
pub struct DaemonSession {
    /// Session name
    pub name: String,
    /// Device ID
    pub device: Option<String>,
    /// VM Service URL (None = not connected)
    pub vm_url: Option<String>,
    /// VM Service client for Flutter communication
    pub vm_service: VmServiceClient,
    /// Device operator for platform-agnostic device interaction
    pub device_op: Box<dyn DeviceOperator>,
    /// Device platform
    pub platform: Platform,
    /// Reference map for element lookup
    pub ref_map: RefMap,
    /// Track if this session has a flutter process (run mode)
    pub has_flutter_process: bool,
    /// Android package name (native Android mode)
    pub package: Option<String>,
    /// WDA port (iOS mode)
    pub wda_port: Option<u16>,
    /// WDA session ID (iOS mode)
    pub wda_session_id: Option<String>,
    /// Device scale factor (iOS mode)
    pub wda_scale: Option<f64>,
}

impl DaemonSession {
    /// Create a new daemon session (defaults to Android)
    pub fn new(name: &str, device: Option<String>) -> Self {
        info!("Creating session: {} (device: {:?})", name, device);
        let adb = AdbClient::new(device.clone());
        Self {
            name: name.to_string(),
            device,
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op: Box::new(adb),
            platform: Platform::Android,
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
            wda_port: None,
            wda_session_id: None,
            wda_scale: None,
        }
    }

    /// Create a new iOS daemon session
    pub fn new_ios(name: &str, device: Option<String>, device_op: Box<dyn DeviceOperator>) -> Self {
        info!("Creating iOS session: {} (device: {:?})", name, device);
        Self {
            name: name.to_string(),
            device,
            vm_url: None,
            vm_service: VmServiceClient::new(),
            device_op,
            platform: Platform::IOS,
            ref_map: RefMap::new(),
            has_flutter_process: false,
            package: None,
            wda_port: None,
            wda_session_id: None,
            wda_scale: None,
        }
    }

    /// Check if the session is connected to a VM Service
    pub fn is_connected(&self) -> bool {
        self.vm_url.is_some() || self.package.is_some()
    }

    /// Check if this is a native Android session (ADB-only, no VM Service)
    pub fn is_android_mode(&self) -> bool {
        self.package.is_some() && self.vm_url.is_none()
    }
}

/// Manages multiple daemon sessions
pub struct SessionManager {
    sessions: HashMap<String, DaemonSession>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Get an existing session or create a new one
    pub fn get_or_create(&mut self, name: &str, device: Option<String>) -> &mut DaemonSession {
        if !self.sessions.contains_key(name) {
            info!("Creating new session: {}", name);
            let session = DaemonSession::new(name, device);
            self.sessions.insert(name.to_string(), session);
        }
        self.sessions.get_mut(name).unwrap()
    }

    /// Get an existing session by name
    pub fn get(&self, name: &str) -> Option<&DaemonSession> {
        self.sessions.get(name)
    }

    /// Get a mutable reference to an existing session by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut DaemonSession> {
        self.sessions.get_mut(name)
    }

    /// Remove a session by name
    pub fn remove(&mut self, name: &str) -> Option<DaemonSession> {
        info!("Removing session: {}", name);
        self.sessions.remove(name)
    }

    /// List all session names
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_daemon_session_new() {
        let session = DaemonSession::new("test", None);
        assert_eq!(session.name, "test");
        assert!(session.device.is_none());
        assert!(session.vm_url.is_none());
        assert!(!session.is_connected());
    }

    #[test]
    fn test_daemon_session_with_device() {
        let session = DaemonSession::new("test", Some("emulator-5554".to_string()));
        assert_eq!(session.name, "test");
        assert_eq!(session.device, Some("emulator-5554".to_string()));
        assert!(!session.is_connected());
    }

    #[test]
    fn test_session_manager_new() {
        let manager = SessionManager::new();
        assert!(manager.list().is_empty());
    }

    #[test]
    fn test_session_manager_get_or_create() {
        let mut manager = SessionManager::new();

        // First call creates the session
        let session = manager.get_or_create("test", None);
        assert_eq!(session.name, "test");

        // Second call returns the same session
        let session = manager.get_or_create("test", Some("device".to_string()));
        assert_eq!(session.name, "test");
        // Device should still be None since session already existed
        assert!(session.device.is_none());

        assert_eq!(manager.list().len(), 1);
    }

    #[test]
    fn test_session_manager_get() {
        let mut manager = SessionManager::new();

        assert!(manager.get("nonexistent").is_none());

        manager.get_or_create("test", None);
        assert!(manager.get("test").is_some());
    }

    #[test]
    fn test_session_manager_get_mut() {
        let mut manager = SessionManager::new();

        assert!(manager.get_mut("nonexistent").is_none());

        manager.get_or_create("test", None);
        let session = manager.get_mut("test").unwrap();
        session.vm_url = Some("ws://localhost:12345/ws".to_string());

        assert!(manager.get("test").unwrap().is_connected());
    }

    #[test]
    fn test_session_manager_remove() {
        let mut manager = SessionManager::new();

        manager.get_or_create("test", None);
        assert_eq!(manager.list().len(), 1);

        let removed = manager.remove("test");
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().name, "test");
        assert!(manager.list().is_empty());

        // Removing again returns None
        assert!(manager.remove("test").is_none());
    }

    #[test]
    fn test_session_manager_list() {
        let mut manager = SessionManager::new();

        manager.get_or_create("session1", None);
        manager.get_or_create("session2", None);
        manager.get_or_create("session3", None);

        let list = manager.list();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"session1"));
        assert!(list.contains(&"session2"));
        assert!(list.contains(&"session3"));
    }

    #[test]
    fn test_session_manager_default() {
        let manager = SessionManager::default();
        assert!(manager.list().is_empty());
    }
}
