use crate::core::types::{Bounds, ElementNode, ElementRef, RefMap};
use std::collections::HashMap;
use tracing::debug;

/// Fetch and parse the iOS accessibility element tree via WDA
pub fn fetch_element_tree(
    base_url: &str,
    session_id: &str,
    scale: f64,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Result<ElementNode, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    let url = format!("{}/session/{}/source?format=json", base_url, session_id);
    let resp = client.get(&url).send()?;
    let data: serde_json::Value = resp.json()?;

    let source = data.get("value").ok_or("No 'value' in /source response")?;

    let tree = parse_wda_element(source, scale, ref_map, interactive_only, 0);
    tree.ok_or_else(|| "Failed to parse element tree".into())
}

/// Parse a single WDA element node recursively
fn parse_wda_element(
    node: &serde_json::Value,
    scale: f64,
    ref_map: &mut RefMap,
    interactive_only: bool,
    depth: u32,
) -> Option<ElementNode> {
    let element_type_raw = node
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Other");

    // Map XCUIElementType* to simplified types
    let element_type = map_element_type(element_type_raw);

    let label = node
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let name = node
        .get("name")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let value = node
        .get("value")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    // Parse bounds from rect
    let bounds = parse_rect(node, scale);

    let is_interactive = is_interactive_type(&element_type);

    // Parse children first
    let children_raw = node
        .get("children")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    let children: Vec<ElementNode> = children_raw
        .iter()
        .filter_map(|child| parse_wda_element(child, scale, ref_map, interactive_only, depth + 1))
        .collect();

    // Skip non-interactive leaf nodes if filtering
    if interactive_only && !is_interactive && children.is_empty() {
        return None;
    }

    // Skip containers with no label and no interactive children
    if interactive_only && !is_interactive && label.is_none() && children.is_empty() {
        return None;
    }

    // Build display label (prefer label, fallback to name, then value)
    let display_label = label.clone().or_else(|| name.clone()).or_else(|| value.clone());

    // Build properties
    let mut properties = HashMap::new();
    if let Some(ref n) = name {
        properties.insert("name".to_string(), serde_json::Value::String(n.clone()));
    }
    if let Some(ref v) = value {
        properties.insert("value".to_string(), serde_json::Value::String(v.clone()));
    }
    let enabled = node
        .get("enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    if !enabled {
        properties.insert("enabled".to_string(), serde_json::Value::Bool(false));
    }

    // Create ElementRef for ref_map
    let ref_id = if let Some(ref b) = bounds {
        let elem_ref = ElementRef {
            ref_id: String::new(), // set by ref_map.add()
            element_type: element_type.clone(),
            label: display_label.clone(),
            bounds: b.clone(),
            properties: properties.clone(),
            style: None,
        };
        ref_map.add(elem_ref)
    } else {
        format!("e_skip_{}", depth)
    };

    Some(ElementNode {
        ref_id,
        element_type,
        label: display_label,
        bounds,
        properties,
        style: None,
        children,
    })
}

/// Parse rect from WDA element node, converting to physical pixels
fn parse_rect(node: &serde_json::Value, scale: f64) -> Option<Bounds> {
    let rect = node.get("rect")?;
    let x = rect.get("x")?.as_f64()?;
    let y = rect.get("y")?.as_f64()?;
    let width = rect.get("width")?.as_f64()?;
    let height = rect.get("height")?.as_f64()?;

    // WDA returns logical points; convert to physical pixels
    Some(Bounds {
        x: x * scale,
        y: y * scale,
        width: width * scale,
        height: height * scale,
    })
}

/// Map XCUIElementType names to simplified element types
fn map_element_type(raw: &str) -> String {
    let simplified = raw.strip_prefix("XCUIElementType").unwrap_or(raw);
    match simplified {
        "Button" => "Button",
        "StaticText" => "Text",
        "TextField" | "SearchField" => "TextField",
        "SecureTextField" => "SecureTextField",
        "Image" => "Image",
        "Switch" | "Toggle" => "Switch",
        "Slider" => "Slider",
        "ScrollView" => "ScrollView",
        "Table" | "CollectionView" => "List",
        "Cell" => "ListItem",
        "NavigationBar" => "NavigationBar",
        "TabBar" => "TabBar",
        "Alert" => "Alert",
        "Picker" | "PickerWheel" => "Picker",
        "Link" => "Link",
        "CheckBox" => "Checkbox",
        "RadioButton" => "RadioButton",
        "Window" | "Application" => "Container",
        "Other" => "Container",
        other => other,
    }
    .to_string()
}

/// Check if an element type is interactive
fn is_interactive_type(element_type: &str) -> bool {
    matches!(
        element_type,
        "Button"
            | "TextField"
            | "SecureTextField"
            | "Switch"
            | "Slider"
            | "Picker"
            | "Link"
            | "Checkbox"
            | "RadioButton"
            | "ListItem"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_element_type() {
        assert_eq!(map_element_type("XCUIElementTypeButton"), "Button");
        assert_eq!(map_element_type("XCUIElementTypeStaticText"), "Text");
        assert_eq!(map_element_type("XCUIElementTypeTextField"), "TextField");
        assert_eq!(map_element_type("XCUIElementTypeSwitch"), "Switch");
        assert_eq!(map_element_type("XCUIElementTypeTable"), "List");
        assert_eq!(map_element_type("XCUIElementTypeWindow"), "Container");
        assert_eq!(map_element_type("UnknownType"), "UnknownType");
    }

    #[test]
    fn test_is_interactive_type() {
        assert!(is_interactive_type("Button"));
        assert!(is_interactive_type("TextField"));
        assert!(is_interactive_type("Switch"));
        assert!(!is_interactive_type("Text"));
        assert!(!is_interactive_type("Container"));
        assert!(!is_interactive_type("Image"));
    }

    #[test]
    fn test_parse_rect() {
        let node = serde_json::json!({
            "rect": {"x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0}
        });
        let bounds = parse_rect(&node, 3.0).unwrap();
        assert_eq!(bounds.x, 30.0);
        assert_eq!(bounds.y, 60.0);
        assert_eq!(bounds.width, 300.0);
        assert_eq!(bounds.height, 150.0);
    }

    #[test]
    fn test_parse_rect_no_rect() {
        let node = serde_json::json!({"type": "Button"});
        assert!(parse_rect(&node, 3.0).is_none());
    }

    #[test]
    fn test_parse_wda_element_basic() {
        let node = serde_json::json!({
            "type": "XCUIElementTypeButton",
            "label": "Login",
            "rect": {"x": 10.0, "y": 20.0, "width": 100.0, "height": 50.0},
            "enabled": true,
            "children": []
        });

        let mut ref_map = RefMap::new();
        let elem = parse_wda_element(&node, 2.0, &mut ref_map, false, 0).unwrap();
        assert_eq!(elem.element_type, "Button");
        assert_eq!(elem.label, Some("Login".to_string()));
        assert_eq!(elem.bounds.as_ref().unwrap().x, 20.0);
    }
}
