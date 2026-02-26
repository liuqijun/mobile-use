use crate::core::types::{Bounds, ElementNode, ElementRef, RefMap};
use std::collections::HashMap;
use tracing::debug;

use crate::core::Result;
use crate::platform::android::AdbClient;

/// Dump UI hierarchy via UIAutomator and return the raw XML string.
pub fn dump_ui(adb: &AdbClient) -> Result<String> {
    // Dump to device, then cat the result
    let dump_output = adb.shell("uiautomator dump /sdcard/window_dump.xml 2>&1")?;
    debug!("uiautomator dump output: {}", dump_output.trim());

    let xml = adb.shell("cat /sdcard/window_dump.xml")?;

    // Clean up
    let _ = adb.shell("rm /sdcard/window_dump.xml");

    if xml.trim().is_empty() || !xml.contains("<hierarchy") {
        return Err(crate::core::MobileUseError::Other(
            "UIAutomator dump returned empty or invalid XML".to_string(),
        ));
    }

    Ok(xml)
}

/// Parse UIAutomator XML dump into an `ElementNode` tree.
///
/// Populates `ref_map` with interactive elements (clickable, focusable, scrollable, checkable).
/// If `interactive_only` is true, only interactive elements are included in the output tree.
pub fn parse_uiautomator_xml(
    xml: &str,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Option<ElementNode> {
    // Find the hierarchy root
    let hierarchy_start = xml.find("<hierarchy")?;
    let content = &xml[hierarchy_start..];

    // Parse the XML into a tree of raw nodes first
    let raw_nodes = parse_xml_nodes(content);

    if raw_nodes.is_empty() {
        return None;
    }

    // The first node should be <hierarchy>, which contains child <node> elements
    let root_raw = &raw_nodes[0];

    // Convert raw XML tree to ElementNode tree
    convert_to_element_tree(root_raw, ref_map, interactive_only)
}

/// A raw parsed XML node
#[derive(Debug)]
struct RawXmlNode {
    tag: String,
    attrs: HashMap<String, String>,
    children: Vec<RawXmlNode>,
}

/// Simple XML parser for UIAutomator output.
/// UIAutomator XML is well-formed and relatively simple.
fn parse_xml_nodes(xml: &str) -> Vec<RawXmlNode> {
    let mut nodes = Vec::new();
    let mut pos = 0;
    let bytes = xml.as_bytes();

    while pos < bytes.len() {
        // Skip to next '<'
        while pos < bytes.len() && bytes[pos] != b'<' {
            pos += 1;
        }
        if pos >= bytes.len() {
            break;
        }

        // Skip XML declaration and comments
        if xml[pos..].starts_with("<?") || xml[pos..].starts_with("<!--") {
            if let Some(end) = xml[pos..].find('>') {
                pos += end + 1;
                continue;
            }
            break;
        }

        // Skip closing tags at this level
        if xml[pos..].starts_with("</") {
            if let Some(end) = xml[pos..].find('>') {
                pos += end + 1;
            } else {
                break;
            }
            continue;
        }

        // Parse opening tag
        if let Some(node) = parse_single_node(xml, &mut pos) {
            nodes.push(node);
        } else {
            pos += 1;
        }
    }

    nodes
}

/// Parse a single XML node starting at position, advancing pos past it.
fn parse_single_node(xml: &str, pos: &mut usize) -> Option<RawXmlNode> {
    let start = *pos;
    if start >= xml.len() || !xml[start..].starts_with('<') {
        return None;
    }

    // Find end of opening tag
    let tag_end = find_tag_end(xml, start)?;
    let tag_content = &xml[start + 1..tag_end];

    // Check if self-closing
    let self_closing = tag_content.ends_with('/');
    let tag_content = if self_closing {
        &tag_content[..tag_content.len() - 1]
    } else {
        tag_content
    };

    // Parse tag name and attributes
    let (tag_name, attrs) = parse_tag_attrs(tag_content)?;

    *pos = tag_end + 1;

    let mut children = Vec::new();

    if !self_closing {
        // Parse children until closing tag
        let closing_tag = format!("</{}>", tag_name);
        loop {
            // Skip whitespace/text
            while *pos < xml.len() && xml.as_bytes()[*pos] != b'<' {
                *pos += 1;
            }
            if *pos >= xml.len() {
                break;
            }

            // Check for closing tag
            if xml[*pos..].starts_with(&closing_tag) {
                *pos += closing_tag.len();
                break;
            }

            // Skip comments
            if xml[*pos..].starts_with("<!--") {
                if let Some(end) = xml[*pos..].find("-->") {
                    *pos += end + 3;
                    continue;
                }
                break;
            }

            // Skip other closing tags (shouldn't happen in well-formed XML)
            if xml[*pos..].starts_with("</") {
                if let Some(end) = xml[*pos..].find('>') {
                    *pos += end + 1;
                }
                break;
            }

            // Parse child node
            if let Some(child) = parse_single_node(xml, pos) {
                children.push(child);
            } else {
                *pos += 1;
            }
        }
    }

    Some(RawXmlNode {
        tag: tag_name.to_string(),
        attrs,
        children,
    })
}

/// Find the position of '>' that closes an opening tag, handling quoted attributes.
fn find_tag_end(xml: &str, start: usize) -> Option<usize> {
    let bytes = xml.as_bytes();
    let mut i = start + 1;
    let mut in_quote = false;
    let mut quote_char = b'"';

    while i < bytes.len() {
        if in_quote {
            if bytes[i] == quote_char {
                in_quote = false;
            }
        } else {
            match bytes[i] {
                b'"' | b'\'' => {
                    in_quote = true;
                    quote_char = bytes[i];
                }
                b'>' => return Some(i),
                _ => {}
            }
        }
        i += 1;
    }
    None
}

/// Parse tag name and attributes from tag content string.
fn parse_tag_attrs(content: &str) -> Option<(String, HashMap<String, String>)> {
    let content = content.trim();
    let mut attrs = HashMap::new();

    // Split tag name from rest
    let (tag_name, rest) = if let Some(space_pos) = content.find(|c: char| c.is_whitespace()) {
        (&content[..space_pos], &content[space_pos..])
    } else {
        (content, "")
    };

    if tag_name.is_empty() {
        return None;
    }

    // Parse attributes with regex
    let attr_re = regex::Regex::new(r#"(\w[\w-]*)="([^"]*)""#).ok()?;
    for caps in attr_re.captures_iter(rest) {
        let key = caps[1].to_string();
        let value = unescape_xml(&caps[2]);
        attrs.insert(key, value);
    }

    Some((tag_name.to_string(), attrs))
}

/// Unescape basic XML entities.
fn unescape_xml(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// Convert a raw XML node tree into an ElementNode tree.
fn convert_to_element_tree(
    raw: &RawXmlNode,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Option<ElementNode> {
    if raw.tag == "hierarchy" {
        // Hierarchy is the root container
        let mut children = Vec::new();
        for child in &raw.children {
            if let Some(node) = convert_to_element_tree(child, ref_map, interactive_only) {
                children.push(node);
            }
        }
        if children.is_empty() {
            return None;
        }

        return Some(ElementNode {
            ref_id: String::new(),
            element_type: "Screen".to_string(),
            label: None,
            bounds: None,
            properties: HashMap::new(),
            style: None,
            children,
        });
    }

    if raw.tag != "node" {
        return None;
    }

    let class = raw.attrs.get("class").cloned().unwrap_or_default();
    let text = raw.attrs.get("text").cloned().unwrap_or_default();
    let content_desc = raw.attrs.get("content-desc").cloned().unwrap_or_default();
    let resource_id = raw.attrs.get("resource-id").cloned().unwrap_or_default();
    let bounds_str = raw.attrs.get("bounds").cloned().unwrap_or_default();

    let clickable = raw.attrs.get("clickable").map(|v| v == "true").unwrap_or(false);
    let focusable = raw.attrs.get("focusable").map(|v| v == "true").unwrap_or(false);
    let scrollable = raw.attrs.get("scrollable").map(|v| v == "true").unwrap_or(false);
    let checkable = raw.attrs.get("checkable").map(|v| v == "true").unwrap_or(false);
    let checked = raw.attrs.get("checked").map(|v| v == "true").unwrap_or(false);
    let enabled = raw.attrs.get("enabled").map(|v| v == "true").unwrap_or(true);
    let selected = raw.attrs.get("selected").map(|v| v == "true").unwrap_or(false);
    let long_clickable = raw.attrs.get("long-clickable").map(|v| v == "true").unwrap_or(false);

    let is_interactive = clickable || focusable || scrollable || checkable;

    // Parse bounds
    let bounds = parse_bounds(&bounds_str);

    // Map Android class to element type
    let element_type = map_class_to_type(&class);

    // Determine label
    let label = if !text.is_empty() {
        Some(text.clone())
    } else if !content_desc.is_empty() {
        Some(content_desc.clone())
    } else {
        None
    };

    // Build properties
    let mut properties = HashMap::new();
    if clickable {
        properties.insert("clickable".to_string(), serde_json::Value::Bool(true));
    }
    if focusable {
        properties.insert("focusable".to_string(), serde_json::Value::Bool(true));
    }
    if scrollable {
        properties.insert("scrollable".to_string(), serde_json::Value::Bool(true));
    }
    if checkable {
        properties.insert("isCheckable".to_string(), serde_json::Value::Bool(true));
    }
    if checked {
        properties.insert("isChecked".to_string(), serde_json::Value::Bool(true));
    }
    if !enabled {
        properties.insert("isDisabled".to_string(), serde_json::Value::Bool(true));
    }
    if selected {
        properties.insert("isSelected".to_string(), serde_json::Value::Bool(true));
    }
    if long_clickable {
        properties.insert("longClickable".to_string(), serde_json::Value::Bool(true));
    }
    if !resource_id.is_empty() {
        properties.insert(
            "resourceId".to_string(),
            serde_json::Value::String(resource_id),
        );
    }
    if !class.is_empty() {
        properties.insert("class".to_string(), serde_json::Value::String(class));
    }

    // Recurse into children
    let mut children = Vec::new();
    for child in &raw.children {
        if let Some(child_node) = convert_to_element_tree(child, ref_map, interactive_only) {
            children.push(child_node);
        }
    }

    // If interactive_only, skip non-interactive leaf nodes
    if interactive_only && !is_interactive && children.is_empty() {
        return None;
    }

    // If interactive_only and not interactive but has children, keep as container
    // but don't assign a ref
    let ref_id = if is_interactive {
        if let Some(ref b) = bounds {
            let element_ref = ElementRef {
                ref_id: String::new(),
                element_type: element_type.clone(),
                label: label.clone(),
                bounds: b.clone(),
                properties: properties.clone(),
                style: None,
            };
            ref_map.add(element_ref)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    // If interactive_only and this is a non-interactive container with children,
    // still include it to preserve tree structure
    if interactive_only && !is_interactive && !children.is_empty() {
        return Some(ElementNode {
            ref_id: String::new(),
            element_type,
            label,
            bounds,
            properties,
            style: None,
            children,
        });
    }

    Some(ElementNode {
        ref_id,
        element_type,
        label,
        bounds,
        properties,
        style: None,
        children,
    })
}

/// Parse UIAutomator bounds string "[left,top][right,bottom]" into Bounds.
fn parse_bounds(bounds_str: &str) -> Option<Bounds> {
    // Format: [left,top][right,bottom]
    let re = regex::Regex::new(r"\[(\d+),(\d+)\]\[(\d+),(\d+)\]").ok()?;
    let caps = re.captures(bounds_str)?;

    let left: f64 = caps[1].parse().ok()?;
    let top: f64 = caps[2].parse().ok()?;
    let right: f64 = caps[3].parse().ok()?;
    let bottom: f64 = caps[4].parse().ok()?;

    Some(Bounds {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

/// Map Android class names to simplified element types.
fn map_class_to_type(class: &str) -> String {
    // Strip package prefix for matching
    let short = class.rsplit('.').next().unwrap_or(class);

    match short {
        "Button" | "MaterialButton" | "AppCompatButton" => "Button".to_string(),
        "ImageButton" | "FloatingActionButton" | "ExtendedFloatingActionButton" => {
            "ImageButton".to_string()
        }
        "EditText" | "AppCompatEditText" | "TextInputEditText" => "TextField".to_string(),
        "TextView" | "AppCompatTextView" | "MaterialTextView" => "Text".to_string(),
        "ImageView" | "AppCompatImageView" | "ShapeableImageView" => "Image".to_string(),
        "CheckBox" | "AppCompatCheckBox" | "MaterialCheckBox" => "CheckBox".to_string(),
        "Switch" | "SwitchCompat" | "SwitchMaterial" => "Switch".to_string(),
        "RadioButton" | "AppCompatRadioButton" | "MaterialRadioButton" => {
            "RadioButton".to_string()
        }
        "Spinner" | "AppCompatSpinner" => "Spinner".to_string(),
        "SeekBar" | "AppCompatSeekBar" | "Slider" => "Slider".to_string(),
        "ProgressBar" => "ProgressBar".to_string(),
        "RecyclerView" | "ListView" | "ScrollView" | "NestedScrollView"
        | "HorizontalScrollView" => "ScrollView".to_string(),
        "ViewPager" | "ViewPager2" => "ViewPager".to_string(),
        "TabLayout" | "TabItem" => "Tab".to_string(),
        "Toolbar" | "MaterialToolbar" => "Toolbar".to_string(),
        "NavigationView" | "BottomNavigationView" | "NavigationBarView" => "Navigation".to_string(),
        "CardView" | "MaterialCardView" => "Card".to_string(),
        "ChipGroup" | "Chip" => "Chip".to_string(),
        "WebView" => "WebView".to_string(),
        "View" => "View".to_string(),
        "FrameLayout" | "LinearLayout" | "RelativeLayout" | "ConstraintLayout"
        | "CoordinatorLayout" | "AppBarLayout" => "Layout".to_string(),
        _ => short.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_bounds() {
        let bounds = parse_bounds("[0,0][1080,1920]").unwrap();
        assert_eq!(bounds.x, 0.0);
        assert_eq!(bounds.y, 0.0);
        assert_eq!(bounds.width, 1080.0);
        assert_eq!(bounds.height, 1920.0);

        let bounds = parse_bounds("[100,200][400,500]").unwrap();
        assert_eq!(bounds.x, 100.0);
        assert_eq!(bounds.y, 200.0);
        assert_eq!(bounds.width, 300.0);
        assert_eq!(bounds.height, 300.0);

        assert!(parse_bounds("invalid").is_none());
    }

    #[test]
    fn test_map_class_to_type() {
        assert_eq!(map_class_to_type("android.widget.Button"), "Button");
        assert_eq!(
            map_class_to_type("android.widget.EditText"),
            "TextField"
        );
        assert_eq!(map_class_to_type("android.widget.TextView"), "Text");
        assert_eq!(
            map_class_to_type("android.widget.ImageView"),
            "Image"
        );
        assert_eq!(map_class_to_type("android.widget.CheckBox"), "CheckBox");
        assert_eq!(
            map_class_to_type("com.google.android.material.button.MaterialButton"),
            "Button"
        );
        assert_eq!(map_class_to_type("android.view.View"), "View");
        assert_eq!(
            map_class_to_type("android.widget.FrameLayout"),
            "Layout"
        );
        assert_eq!(
            map_class_to_type("com.example.CustomWidget"),
            "CustomWidget"
        );
    }

    #[test]
    fn test_parse_uiautomator_xml_basic() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hierarchy rotation="0">
  <node index="0" text="" resource-id="" class="android.widget.FrameLayout" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,0][1080,1920]">
    <node index="0" text="Hello" resource-id="com.example:id/greeting" class="android.widget.TextView" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[100,200][400,260]" />
    <node index="1" text="Click Me" resource-id="com.example:id/button" class="android.widget.Button" package="com.example" content-desc="" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[200,400][500,480]" />
  </node>
</hierarchy>"#;

        let mut ref_map = RefMap::new();
        let tree = parse_uiautomator_xml(xml, &mut ref_map, false).unwrap();

        assert_eq!(tree.element_type, "Screen");
        assert_eq!(tree.children.len(), 1); // FrameLayout

        let frame = &tree.children[0];
        assert_eq!(frame.element_type, "Layout");
        assert_eq!(frame.children.len(), 2);

        let text_node = &frame.children[0];
        assert_eq!(text_node.element_type, "Text");
        assert_eq!(text_node.label, Some("Hello".to_string()));

        let button_node = &frame.children[1];
        assert_eq!(button_node.element_type, "Button");
        assert_eq!(button_node.label, Some("Click Me".to_string()));
        assert!(!button_node.ref_id.is_empty()); // Should have a ref (clickable)

        // Verify ref_map
        assert_eq!(ref_map.refs.len(), 1); // Only the button is interactive
        let btn_ref = ref_map.get(&button_node.ref_id).unwrap();
        assert_eq!(btn_ref.element_type, "Button");
        assert_eq!(btn_ref.bounds.x, 200.0);
        assert_eq!(btn_ref.bounds.y, 400.0);
    }

    #[test]
    fn test_parse_uiautomator_xml_interactive_only() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<hierarchy rotation="0">
  <node index="0" text="" class="android.widget.FrameLayout" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,0][1080,1920]">
    <node index="0" text="Static" class="android.widget.TextView" package="com.example" content-desc="" checkable="false" checked="false" clickable="false" enabled="true" focusable="false" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,0][100,50]" />
    <node index="1" text="Click" class="android.widget.Button" package="com.example" content-desc="" checkable="false" checked="false" clickable="true" enabled="true" focusable="true" focused="false" scrollable="false" long-clickable="false" password="false" selected="false" bounds="[0,100][200,150]" />
  </node>
</hierarchy>"#;

        let mut ref_map = RefMap::new();
        let tree = parse_uiautomator_xml(xml, &mut ref_map, true).unwrap();

        // The FrameLayout should still appear as container, but static TextView should be filtered
        let frame = &tree.children[0];
        assert_eq!(frame.children.len(), 1); // Only the button
        assert_eq!(frame.children[0].element_type, "Button");
    }

    #[test]
    fn test_unescape_xml() {
        assert_eq!(unescape_xml("hello &amp; world"), "hello & world");
        assert_eq!(unescape_xml("a &lt; b &gt; c"), "a < b > c");
        assert_eq!(unescape_xml("say &quot;hi&quot;"), "say \"hi\"");
    }
}
