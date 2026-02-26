use crate::core::{MobileUseError, Result};
use crate::platform::android::AdbClient;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tracing::{debug, info};

/// A detected Gradle app module
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct GradleModule {
    /// Module name (e.g., "app-compose")
    pub name: String,
    /// Module directory path relative to project root
    pub path: PathBuf,
    /// Detected application ID / package name
    pub package: Option<String>,
}

/// Find Gradle modules that have the `com.android.application` plugin.
///
/// Parses `settings.gradle.kts` (or `settings.gradle`) for `include(...)` directives,
/// then checks each module's `build.gradle.kts`/`build.gradle` for the application plugin.
pub fn find_gradle_modules(project_dir: &Path) -> Result<Vec<GradleModule>> {
    let settings_path = if project_dir.join("settings.gradle.kts").exists() {
        project_dir.join("settings.gradle.kts")
    } else if project_dir.join("settings.gradle").exists() {
        project_dir.join("settings.gradle")
    } else {
        return Err(MobileUseError::Other(
            "No settings.gradle(.kts) found".to_string(),
        ));
    };

    let settings_content = std::fs::read_to_string(&settings_path).map_err(|e| {
        MobileUseError::Other(format!("Failed to read {}: {}", settings_path.display(), e))
    })?;

    let mut modules = Vec::new();

    // Collect module names from both Kotlin DSL and Groovy syntax
    let mut module_refs = Vec::new();

    // Kotlin DSL: include(":app") or include(":app", ":lib")
    let kts_re = regex::Regex::new(r#"include\s*\([^)]*\)"#).unwrap();
    let kts_module_re = regex::Regex::new(r#""([^"]+)""#).unwrap();
    for m in kts_re.find_iter(&settings_content) {
        for caps in kts_module_re.captures_iter(m.as_str()) {
            module_refs.push(caps[1].to_string());
        }
    }

    // Groovy: include ':app', ':call-ui' or include ':app'
    let groovy_re = regex::Regex::new(r#"include\s+('[^)]*)"#).unwrap();
    let groovy_module_re = regex::Regex::new(r#"'([^']+)'"#).unwrap();
    for m in groovy_re.find_iter(&settings_content) {
        for caps in groovy_module_re.captures_iter(m.as_str()) {
            module_refs.push(caps[1].to_string());
        }
    }

    for module_ref in &module_refs {
        let module_name = module_ref.trim_start_matches(':');
        let module_dir = project_dir.join(module_name);

        if !module_dir.exists() {
            debug!("Module directory not found: {}", module_dir.display());
            continue;
        }

        // Check if this module has the application plugin
        let build_file = if module_dir.join("build.gradle.kts").exists() {
            module_dir.join("build.gradle.kts")
        } else if module_dir.join("build.gradle").exists() {
            module_dir.join("build.gradle")
        } else {
            continue;
        };

        let build_content = std::fs::read_to_string(&build_file).unwrap_or_default();
        if !build_content.contains("com.android.application") {
            debug!("Module {} is not an application module", module_name);
            continue;
        }

        let package = detect_package_name(&build_content);

        modules.push(GradleModule {
            name: module_name.to_string(),
            path: module_dir,
            package,
        });
    }

    Ok(modules)
}

/// Extract `applicationId` from a build.gradle(.kts) file content.
fn detect_package_name(build_content: &str) -> Option<String> {
    // Match: applicationId = "com.example.app" or applicationId "com.example.app"
    let re = regex::Regex::new(r#"applicationId\s*[=]?\s*"([^"]+)""#).ok()?;
    re.captures(build_content)
        .map(|caps| caps[1].to_string())
}

/// Build APK using Gradle, streaming output to stdout.
///
/// Runs `./gradlew :module:assembleDebug` and returns the path to the built APK.
pub fn build_apk(project_dir: &Path, module: &GradleModule) -> Result<PathBuf> {
    let gradlew = if cfg!(target_os = "windows") {
        project_dir.join("gradlew.bat")
    } else {
        project_dir.join("gradlew")
    };

    if !gradlew.exists() {
        return Err(MobileUseError::Other(format!(
            "Gradle wrapper not found at {}",
            gradlew.display()
        )));
    }

    // Ensure gradlew has execute permission
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&gradlew)
            .map_err(|e| MobileUseError::Other(format!("Failed to read gradlew metadata: {}", e)))?;
        let mut perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            debug!("Adding execute permission to {}", gradlew.display());
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(&gradlew, perms).map_err(|e| {
                MobileUseError::Other(format!("Failed to set gradlew permissions: {}", e))
            })?;
        }
    }

    let task = format!(":{}:assembleDebug", module.name);
    info!("Building APK: {} {}", gradlew.display(), task);

    let status = std::process::Command::new(&gradlew)
        .arg(&task)
        .current_dir(project_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|e| MobileUseError::Other(format!("Failed to run gradlew: {}", e)))?;

    if !status.success() {
        return Err(MobileUseError::Other(format!(
            "Gradle build failed with exit code: {:?}",
            status.code()
        )));
    }

    find_apk_path(project_dir, module)
}

/// Locate the built debug APK for a module.
fn find_apk_path(project_dir: &Path, module: &GradleModule) -> Result<PathBuf> {
    let apk_dir = project_dir
        .join(&module.name)
        .join("build")
        .join("outputs")
        .join("apk")
        .join("debug");

    if !apk_dir.exists() {
        return Err(MobileUseError::Other(format!(
            "APK output directory not found: {}",
            apk_dir.display()
        )));
    }

    // Look for *-debug.apk
    let entries = std::fs::read_dir(&apk_dir)
        .map_err(|e| MobileUseError::Other(format!("Failed to read APK dir: {}", e)))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.ends_with("-debug.apk") {
                info!("Found APK: {}", path.display());
                return Ok(path);
            }
        }
    }

    Err(MobileUseError::Other(format!(
        "No debug APK found in {}",
        apk_dir.display()
    )))
}

/// Install an APK on the device via ADB.
pub fn install_apk(adb: &AdbClient, apk_path: &Path) -> Result<()> {
    info!("Installing APK: {}", apk_path.display());
    let path_str = apk_path.to_string_lossy();
    adb.install(&path_str)
}

/// Launch an app by package name using `monkey` command.
pub fn launch_app(adb: &AdbClient, package: &str) -> Result<()> {
    info!("Launching app: {}", package);
    adb.shell(&format!(
        "monkey -p {} -c android.intent.category.LAUNCHER 1",
        package
    ))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_package_name_kts() {
        let content = r#"
android {
    namespace = "com.example.app"
    defaultConfig {
        applicationId = "com.example.mobileusetest.compose"
        minSdk = 24
    }
}
"#;
        assert_eq!(
            detect_package_name(content),
            Some("com.example.mobileusetest.compose".to_string())
        );
    }

    #[test]
    fn test_detect_package_name_groovy() {
        let content = r#"
android {
    defaultConfig {
        applicationId "com.example.app"
        minSdkVersion 24
    }
}
"#;
        assert_eq!(
            detect_package_name(content),
            Some("com.example.app".to_string())
        );
    }

    #[test]
    fn test_detect_package_name_none() {
        let content = r#"
plugins {
    id 'com.android.library'
}
"#;
        assert_eq!(detect_package_name(content), None);
    }
}
