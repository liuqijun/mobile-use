use crate::core::{ActionResult, AppInfo, ElementNode};
use serde::Serialize;

/// Output format mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Human,
    Json,
}

/// Output formatter
pub struct OutputFormatter {
    mode: OutputMode,
}

impl OutputFormatter {
    pub fn new(json: bool) -> Self {
        Self {
            mode: if json {
                OutputMode::Json
            } else {
                OutputMode::Human
            },
        }
    }

    /// Check if output mode is human-readable (not JSON)
    pub fn is_human(&self) -> bool {
        self.mode == OutputMode::Human
    }

    /// Print success message
    pub fn success(&self, message: &str) {
        if self.mode == OutputMode::Human {
            println!("{}", message);
        } else {
            self.print_json(&serde_json::json!({
                "success": true,
                "message": message
            }));
        }
    }

    /// Print error message
    pub fn error(&self, message: &str) {
        if self.mode == OutputMode::Human {
            eprintln!("Error: {}", message);
        } else {
            self.print_json(&serde_json::json!({
                "success": false,
                "error": message
            }));
        }
    }

    /// Print action result
    pub fn action_result(&self, result: &ActionResult) {
        if self.mode == OutputMode::Human {
            if result.success {
                if let Some(ref msg) = result.message {
                    println!("{}", msg);
                } else {
                    println!("OK");
                }
            } else {
                eprintln!("Failed: {:?}", result.message);
            }
        } else {
            self.print_json(result);
        }
    }

    /// Print app info
    #[allow(dead_code)]
    pub fn app_info(&self, info: &AppInfo) {
        if self.mode == OutputMode::Human {
            println!("Platform: {}", info.platform);
            if let Some(ref device) = info.device {
                println!("Device: {}", device);
            }
            if let Some(ref app_id) = info.app_id {
                println!("App: {}", app_id);
            }
            if let Some(ref url) = info.vm_service_url {
                println!("VM Service: {}", url);
            }
            println!("Connected: {}", info.connected);
        } else {
            self.print_json(info);
        }
    }

    /// Print element tree
    pub fn element_tree(&self, root: &ElementNode, refs: &serde_json::Value) {
        if self.mode == OutputMode::Human {
            self.print_element_human(root, 0);
        } else {
            self.print_json(&serde_json::json!({
                "success": true,
                "data": {
                    "tree": root,
                    "refs": refs
                }
            }));
        }
    }

    /// Print element in human-readable format
    fn print_element_human(&self, node: &ElementNode, indent: usize) {
        let prefix = "  ".repeat(indent);
        let mut line = format!("{}- {} ", prefix, node.element_type);

        if let Some(ref label) = node.label {
            line.push_str(&format!("\"{}\" ", label));
        }

        line.push_str(&format!("[ref=@{}]", node.ref_id));

        // Add bounds info
        if let Some(ref bounds) = node.bounds {
            line.push_str(&format!(
                " ({:.0},{:.0} {:.0}x{:.0})",
                bounds.x, bounds.y, bounds.width, bounds.height
            ));
        }

        // Add style info if present
        if let Some(ref style) = node.style {
            let mut style_parts = Vec::new();

            if let Some(ref bg) = style.background_color {
                style_parts.push(format!("bg:{}", bg));
            }
            if let Some(ref tc) = style.text_color {
                style_parts.push(format!("color:{}", tc));
            }
            if let Some(fs) = style.font_size {
                style_parts.push(format!("font:{:.0}px", fs));
            }
            if let Some(ref fw) = style.font_weight {
                style_parts.push(format!("weight:{}", fw));
            }
            if let Some(br) = style.border_radius {
                style_parts.push(format!("radius:{:.0}", br));
            }

            if !style_parts.is_empty() {
                line.push_str(&format!(" {{{}}}", style_parts.join(" ")));
            }
        }

        // Add key properties (only important ones)
        for (key, value) in &node.properties {
            // Only show important properties
            if key == "isButton" || key == "isEnabled" || key == "isHeader" {
                if let Some(b) = value.as_bool() {
                    if b {
                        line.push_str(&format!(" [{}]", key));
                    }
                }
            }
        }

        println!("{}", line);

        for child in &node.children {
            self.print_element_human(child, indent + 1);
        }
    }

    /// Print raw text
    pub fn raw(&self, text: &str) {
        if self.mode == OutputMode::Human {
            println!("{}", text);
        } else {
            self.print_json(&serde_json::json!({
                "success": true,
                "data": text
            }));
        }
    }

    /// Print informational message (always text, not wrapped in JSON success)
    pub fn info(&self, message: &str) {
        if self.mode == OutputMode::Human {
            println!("{}", message);
        } else {
            self.print_json(&serde_json::json!({
                "info": message
            }));
        }
    }

    /// Print raw JSON value (for daemon responses)
    pub fn json<T: Serialize>(&self, value: &T) {
        self.print_json(value);
    }

    /// Print JSON value
    fn print_json<T: Serialize>(&self, value: &T) {
        if let Ok(json) = serde_json::to_string_pretty(value) {
            println!("{}", json);
        }
    }
}
