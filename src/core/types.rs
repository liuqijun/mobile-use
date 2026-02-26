use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGBA Color
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: f64,
}

impl Color {
    pub fn to_hex(&self) -> String {
        if (self.a - 1.0).abs() < 0.01 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!(
                "#{:02X}{:02X}{:02X}{:02X}",
                self.r,
                self.g,
                self.b,
                (self.a * 255.0) as u8
            )
        }
    }
}

/// Style information for visual comparison
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub elevation: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub widget_type: Option<String>,
}

impl StyleInfo {
    pub fn is_empty(&self) -> bool {
        self.background_color.is_none()
            && self.text_color.is_none()
            && self.font_size.is_none()
            && self.font_weight.is_none()
            && self.border_radius.is_none()
            && self.elevation.is_none()
            && self.padding.is_none()
            && self.widget_type.is_none()
    }
}

/// Element bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Bounds {
    pub fn center(&self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

/// Element reference info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementRef {
    pub ref_id: String,
    pub element_type: String,
    pub label: Option<String>,
    pub bounds: Bounds,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleInfo>,
}

/// Element tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementNode {
    pub ref_id: String,
    pub element_type: String,
    pub label: Option<String>,
    pub bounds: Option<Bounds>,
    pub properties: HashMap<String, serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<StyleInfo>,
    pub children: Vec<ElementNode>,
}

/// Ref map for element lookup
#[derive(Debug, Default)]
pub struct RefMap {
    pub refs: HashMap<String, ElementRef>,
    counter: u32,
}

impl RefMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a RefMap with existing refs and a specific counter value
    pub fn with_refs(refs: HashMap<String, ElementRef>, counter: u32) -> Self {
        Self { refs, counter }
    }

    pub fn clear(&mut self) {
        self.refs.clear();
        self.counter = 0;
    }

    pub fn add(&mut self, mut element: ElementRef) -> String {
        self.counter += 1;
        let ref_id = format!("e{}", self.counter);
        element.ref_id = ref_id.clone(); // Set the ref_id on the element
        self.refs.insert(ref_id.clone(), element);
        ref_id
    }

    pub fn get(&self, ref_id: &str) -> Option<&ElementRef> {
        // Handle @e1 format
        let id = ref_id.trim_start_matches('@');
        self.refs.get(id)
    }
}

/// App connection info
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub platform: String,
    pub device: Option<String>,
    pub app_id: Option<String>,
    pub vm_service_url: Option<String>,
    pub connected: bool,
}

/// Scroll/swipe direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl std::str::FromStr for Direction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "up" => Ok(Direction::Up),
            "down" => Ok(Direction::Down),
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            _ => Err(format!("Invalid direction: {}", s)),
        }
    }
}

/// Action to execute
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Action {
    Tap {
        ref_id: String,
    },
    DoubleTap {
        ref_id: String,
    },
    LongPress {
        ref_id: String,
        duration_ms: u32,
    },
    Input {
        ref_id: String,
        text: String,
        clear: bool,
    },
    Clear {
        ref_id: String,
    },
    Scroll {
        direction: Direction,
        distance: i32,
    },
    ScrollTo {
        ref_id: String,
    },
    Swipe {
        direction: Direction,
        from_ref: Option<String>,
    },
    Wait {
        target: WaitTarget,
        timeout_ms: u32,
    },
}

/// Wait target
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum WaitTarget {
    Element(String),
    Text(String),
    Duration(u32),
}

/// Action result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
    pub data: Option<serde_json::Value>,
}
