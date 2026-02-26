use crate::core::{Bounds, Color, ElementNode, ElementRef, RefMap, StyleInfo};
use regex::Regex;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// Parse Flutter semantics tree into element tree
/// The data is expected to be a JSON object with a "data" field containing text-formatted tree
pub fn parse_semantics_tree(
    data: &Value,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Option<ElementNode> {
    ref_map.clear();

    // Extract text data from the response
    let text = if let Some(text_data) = data.get("data").and_then(|v| v.as_str()) {
        text_data.to_string()
    } else if let Some(text_data) = data.as_str() {
        text_data.to_string()
    } else {
        // Try to use the whole object as before (fallback)
        return parse_node_json(data, ref_map, interactive_only);
    };

    // Parse the text format
    parse_text_tree(&text, ref_map, interactive_only)
}

/// Parse text-formatted semantics tree
fn parse_text_tree(
    text: &str,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Option<ElementNode> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.is_empty() {
        return None;
    }

    // Extract device pixel ratio from "scaled by X.Xx" text
    let scale_factor = extract_scale_factor(&lines).unwrap_or(1.0);

    // Parse nodes from text with scale factor and zero parent offset
    let (root, _) = parse_text_node_scaled(
        &lines,
        0,
        0,
        ref_map,
        interactive_only,
        scale_factor,
        0.0,
        0.0,
    )?;
    Some(root)
}

/// Extract scale factor from semantics text
fn extract_scale_factor(lines: &[&str]) -> Option<f64> {
    for line in lines {
        if line.contains("scaled by") {
            // Parse "scaled by 2.8x" format
            let re = Regex::new(r"scaled by ([\d.]+)x").ok()?;
            if let Some(caps) = re.captures(line) {
                return caps.get(1)?.as_str().parse().ok();
            }
        }
    }
    None
}

/// Parsed node info from text
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct ParsedNodeInfo {
    node_id: String,
    bounds: Option<Bounds>,
    flags: HashSet<String>,
    actions: HashSet<String>,
    label: Option<String>,
    value: Option<String>,
    indent_level: usize,
}

/// Parse a single node and its children from text lines (with scale factor)
/// parent_offset_x/y are the accumulated offsets from ancestor nodes (in logical pixels before scaling)
fn parse_text_node_scaled(
    lines: &[&str],
    start_idx: usize,
    _parent_indent: usize,
    ref_map: &mut RefMap,
    interactive_only: bool,
    scale_factor: f64,
    parent_offset_x: f64,
    parent_offset_y: f64,
) -> Option<(ElementNode, usize)> {
    if start_idx >= lines.len() {
        return None;
    }

    // Parse the header line (SemanticsNode#N)
    let header_line = lines[start_idx];
    let node_info = parse_node_header(header_line)?;
    let node_indent = node_info.indent_level;

    // Parse properties from following lines
    let mut idx = start_idx + 1;
    let mut bounds: Option<Bounds> = None;
    let mut local_bounds: Option<Bounds> = None; // Original logical coordinates for child offset calculation
    let mut flags: HashSet<String> = HashSet::new();
    let mut actions: HashSet<String> = HashSet::new();
    let mut label: Option<String> = None;
    let mut value: Option<String> = None;

    // Parse property lines
    while idx < lines.len() {
        let line = lines[idx];
        let line_indent = get_indent_level(line);

        // If this line is a new node (contains SemanticsNode#), stop parsing properties
        if line.contains("SemanticsNode#") {
            break;
        }

        // If indent is less than or equal to node_indent, we're done with this node
        if line_indent <= node_indent
            && !line.trim().is_empty()
            && !line.trim().starts_with('│')
            && !line.trim().starts_with('├')
            && !line.trim().starts_with('└')
        {
            break;
        }

        // Parse property line
        let trimmed = strip_tree_chars(line);

        // Parse Rect.fromLTRB(...) and apply scale factor with parent offset
        if trimmed.contains("Rect.fromLTRB") {
            if let Some(b) = parse_rect(&trimmed) {
                // Store original logical coordinates for passing to children
                local_bounds = Some(b.clone());

                // Apply parent offset and scale factor to get absolute physical coordinates
                bounds = Some(Bounds {
                    x: (b.x + parent_offset_x) * scale_factor,
                    y: (b.y + parent_offset_y) * scale_factor,
                    width: b.width * scale_factor,
                    height: b.height * scale_factor,
                });
            }
        }
        // Parse flags
        else if trimmed.starts_with("flags:") {
            let flag_str = trimmed.strip_prefix("flags:").unwrap_or("").trim();
            for flag in flag_str.split(',') {
                let flag = flag.trim();
                if !flag.is_empty() {
                    flags.insert(flag.to_string());
                }
            }
        }
        // Parse actions
        else if trimmed.starts_with("actions:") {
            let action_str = trimmed.strip_prefix("actions:").unwrap_or("").trim();
            for action in action_str.split(',') {
                let action = action.trim();
                if !action.is_empty() {
                    actions.insert(action.to_string());
                }
            }
        }
        // Parse label (may be multi-line)
        else if trimmed.starts_with("label:") {
            let label_part = trimmed.strip_prefix("label:").unwrap_or("").trim();
            if label_part.starts_with('"') {
                // Multi-line label
                let mut full_label = label_part.trim_matches('"').to_string();
                idx += 1;
                while idx < lines.len() {
                    let next_line = strip_tree_chars(lines[idx]);
                    if next_line.ends_with('"') {
                        full_label.push_str("\n");
                        full_label.push_str(next_line.trim_end_matches('"'));
                        break;
                    } else if next_line.contains(':') && !next_line.starts_with(' ') {
                        idx -= 1;
                        break;
                    } else {
                        full_label.push_str("\n");
                        full_label.push_str(&next_line);
                    }
                    idx += 1;
                }
                label = Some(full_label);
            } else {
                label = Some(label_part.trim_matches('"').to_string());
            }
        }
        // Parse value
        else if trimmed.starts_with("value:") {
            let value_str = trimmed.strip_prefix("value:").unwrap_or("").trim();
            value = Some(value_str.trim_matches('"').to_string());
        }

        idx += 1;
    }

    // Calculate child offset - children's coordinates are relative to this node's position
    // For scrollable containers (hasImplicitScrolling), child coords are relative to scroll viewport
    let (child_offset_x, child_offset_y) = if let Some(ref lb) = local_bounds {
        // Pass current node's position as offset for children
        (parent_offset_x + lb.x, parent_offset_y + lb.y)
    } else {
        (parent_offset_x, parent_offset_y)
    };

    // Parse children
    let mut children: Vec<ElementNode> = Vec::new();
    while idx < lines.len() {
        let line = lines[idx];

        // Check if this is a child node
        if line.contains("SemanticsNode#") {
            let child_indent = get_indent_level(line);
            if child_indent > node_indent {
                // This is a child - pass accumulated offset
                if let Some((child, new_idx)) = parse_text_node_scaled(
                    lines,
                    idx,
                    node_indent,
                    ref_map,
                    interactive_only,
                    scale_factor,
                    child_offset_x,
                    child_offset_y,
                ) {
                    children.push(child);
                    idx = new_idx;
                    continue;
                }
            } else {
                // This is a sibling or parent's sibling, stop
                break;
            }
        }
        idx += 1;
    }

    // Determine element type
    let element_type = determine_element_type_from_flags(&flags, &actions, &label);
    let is_interactive = is_interactive_from_flags(&element_type, &actions);

    // Build properties map
    let mut properties = HashMap::new();
    if let Some(v) = &value {
        properties.insert("value".to_string(), Value::String(v.clone()));
    }
    for flag in &flags {
        properties.insert(flag.clone(), Value::Bool(true));
    }
    for action in &actions {
        properties.insert(format!("action:{}", action), Value::Bool(true));
    }

    // Skip non-interactive elements if filtering (but keep their children)
    if interactive_only && !is_interactive && label.is_none() {
        return match children.len() {
            0 => None,
            1 => Some((children.into_iter().next().unwrap(), idx)),
            _ => {
                // Create synthetic container for multiple children
                Some((
                    ElementNode {
                        ref_id: "synthetic".to_string(),
                        element_type: "container".to_string(),
                        label: None,
                        bounds: None,
                        properties: HashMap::new(),
                        style: None,
                        children,
                    },
                    idx,
                ))
            }
        };
    }

    // Create ref for this element
    let ref_id = ref_map.add(ElementRef {
        ref_id: String::new(),
        element_type: element_type.clone(),
        label: label.clone(),
        bounds: bounds.clone().unwrap_or(Bounds {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }),
        properties: properties.clone(),
        style: None,
    });

    Some((
        ElementNode {
            ref_id,
            element_type,
            label,
            bounds,
            properties,
            style: None,
            children,
        },
        idx,
    ))
}

/// Parse node header line: "SemanticsNode#N" or with tree prefix
fn parse_node_header(line: &str) -> Option<ParsedNodeInfo> {
    let indent_level = get_indent_level(line);

    // Extract node ID
    let re = Regex::new(r"SemanticsNode#(\d+)").ok()?;
    let caps = re.captures(line)?;
    let node_id = caps.get(1)?.as_str().to_string();

    Some(ParsedNodeInfo {
        node_id,
        bounds: None,
        flags: HashSet::new(),
        actions: HashSet::new(),
        label: None,
        value: None,
        indent_level,
    })
}

/// Get indent level from line (count spaces and tree characters)
fn get_indent_level(line: &str) -> usize {
    let mut level = 0;
    for ch in line.chars() {
        match ch {
            ' ' => level += 1,
            '│' | '├' | '└' | '─' => level += 1,
            _ => break,
        }
    }
    level
}

/// Strip tree drawing characters from line
fn strip_tree_chars(line: &str) -> String {
    let mut result = String::new();
    let mut found_content = false;

    for ch in line.chars() {
        if found_content {
            result.push(ch);
        } else if ch != ' ' && ch != '│' && ch != '├' && ch != '└' && ch != '─' {
            found_content = true;
            result.push(ch);
        }
    }

    result.trim().to_string()
}

/// Parse Rect.fromLTRB(left, top, right, bottom) into Bounds
fn parse_rect(text: &str) -> Option<Bounds> {
    let re = Regex::new(r"Rect\.fromLTRB\(([\d.]+),\s*([\d.]+),\s*([\d.]+),\s*([\d.]+)\)").ok()?;
    let caps = re.captures(text)?;

    let left: f64 = caps.get(1)?.as_str().parse().ok()?;
    let top: f64 = caps.get(2)?.as_str().parse().ok()?;
    let right: f64 = caps.get(3)?.as_str().parse().ok()?;
    let bottom: f64 = caps.get(4)?.as_str().parse().ok()?;

    Some(Bounds {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    })
}

/// Determine element type from flags and actions
fn determine_element_type_from_flags(
    flags: &HashSet<String>,
    actions: &HashSet<String>,
    label: &Option<String>,
) -> String {
    if flags.contains("isButton") || actions.contains("tap") {
        "button".to_string()
    } else if flags.contains("isTextField") || actions.contains("setText") {
        "textField".to_string()
    } else if flags.contains("isChecked") || flags.contains("hasCheckedState") {
        "checkbox".to_string()
    } else if flags.contains("isLink") {
        "link".to_string()
    } else if flags.contains("isImage") {
        "image".to_string()
    } else if flags.contains("isHeader") {
        "header".to_string()
    } else if flags.contains("isSlider") {
        "slider".to_string()
    } else if actions.contains("scrollUp") || actions.contains("scrollDown") {
        "scrollable".to_string()
    } else if label.is_some() {
        "text".to_string()
    } else {
        "container".to_string()
    }
}

/// Check if element is interactive
fn is_interactive_from_flags(element_type: &str, actions: &HashSet<String>) -> bool {
    let interactive_types = [
        "button",
        "textField",
        "checkbox",
        "link",
        "slider",
        "scrollable",
    ];

    if interactive_types.contains(&element_type) {
        return true;
    }

    actions.iter().any(|a| {
        a == "tap" || a == "longPress" || a == "setText" || a == "scrollUp" || a == "scrollDown"
    })
}

// Keep the old JSON parsing as fallback
fn parse_node_json(
    node: &Value,
    ref_map: &mut RefMap,
    interactive_only: bool,
) -> Option<ElementNode> {
    let obj = node.as_object()?;

    // Extract semantics properties
    let label = obj.get("label").and_then(|v| v.as_str()).map(String::from);
    let hint = obj.get("hint").and_then(|v| v.as_str()).map(String::from);
    let value = obj.get("value").and_then(|v| v.as_str()).map(String::from);

    // Extract flags and actions
    let flags = obj.get("flags").and_then(|v| v.as_array());
    let actions = obj.get("actions").and_then(|v| v.as_array());

    // Determine element type from flags and actions
    let element_type = determine_element_type_json(flags, actions, &label);

    // Check if interactive
    let is_interactive = is_interactive_element_json(&element_type, actions);

    // Extract bounds
    let bounds = extract_bounds_json(obj);

    // Build properties map
    let mut properties = HashMap::new();
    if let Some(v) = &value {
        properties.insert("value".to_string(), Value::String(v.clone()));
    }
    if let Some(h) = &hint {
        properties.insert("hint".to_string(), Value::String(h.clone()));
    }
    if let Some(f) = flags {
        for flag in f {
            if let Some(flag_str) = flag.as_str() {
                properties.insert(flag_str.to_string(), Value::Bool(true));
            }
        }
    }

    // Skip non-interactive elements if filtering
    if interactive_only && !is_interactive && label.is_none() {
        if let Some(children) = obj.get("children").and_then(|v| v.as_array()) {
            let matching_children: Vec<ElementNode> = children
                .iter()
                .filter_map(|child| parse_node_json(child, ref_map, interactive_only))
                .collect();

            return match matching_children.len() {
                0 => None,
                1 => matching_children.into_iter().next(),
                _ => Some(ElementNode {
                    ref_id: "synthetic".to_string(),
                    element_type: "container".to_string(),
                    label: None,
                    bounds: None,
                    properties: HashMap::new(),
                    style: None,
                    children: matching_children,
                }),
            };
        }
        return None;
    }

    // Create ref for this element
    let ref_id = ref_map.add(ElementRef {
        ref_id: String::new(),
        element_type: element_type.clone(),
        label: label.clone(),
        bounds: bounds.clone().unwrap_or(Bounds {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        }),
        properties: properties.clone(),
        style: None,
    });

    // Parse children
    let children = if let Some(children_arr) = obj.get("children").and_then(|v| v.as_array()) {
        children_arr
            .iter()
            .filter_map(|child| parse_node_json(child, ref_map, interactive_only))
            .collect()
    } else {
        vec![]
    };

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

fn determine_element_type_json(
    flags: Option<&Vec<Value>>,
    actions: Option<&Vec<Value>>,
    label: &Option<String>,
) -> String {
    let flags_set: HashSet<&str> = flags
        .map(|f| f.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    let actions_set: HashSet<&str> = actions
        .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    if flags_set.contains("isButton") || actions_set.contains("tap") {
        "button".to_string()
    } else if flags_set.contains("isTextField") || actions_set.contains("setText") {
        "textField".to_string()
    } else if flags_set.contains("isChecked") || flags_set.contains("hasCheckedState") {
        "checkbox".to_string()
    } else if flags_set.contains("isLink") {
        "link".to_string()
    } else if flags_set.contains("isImage") {
        "image".to_string()
    } else if flags_set.contains("isHeader") {
        "header".to_string()
    } else if flags_set.contains("isSlider") {
        "slider".to_string()
    } else if actions_set.contains("scrollUp") || actions_set.contains("scrollDown") {
        "scrollable".to_string()
    } else if label.is_some() {
        "text".to_string()
    } else {
        "container".to_string()
    }
}

fn is_interactive_element_json(element_type: &str, actions: Option<&Vec<Value>>) -> bool {
    let interactive_types = [
        "button",
        "textField",
        "checkbox",
        "link",
        "slider",
        "scrollable",
    ];

    if interactive_types.contains(&element_type) {
        return true;
    }

    if let Some(actions) = actions {
        let action_strs: Vec<&str> = actions.iter().filter_map(|v| v.as_str()).collect();
        return action_strs.iter().any(|a| {
            *a == "tap"
                || *a == "longPress"
                || *a == "setText"
                || *a == "scrollUp"
                || *a == "scrollDown"
        });
    }

    false
}

fn extract_bounds_json(obj: &serde_json::Map<String, Value>) -> Option<Bounds> {
    let rect = obj.get("rect")?;
    Some(Bounds {
        x: rect.get("left")?.as_f64()?,
        y: rect.get("top")?.as_f64()?,
        width: rect.get("width")?.as_f64()?,
        height: rect.get("height")?.as_f64()?,
    })
}

// ============= Render Tree Parsing for Style Extraction =============

/// Style info extracted from render tree, indexed by unique characteristics
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RenderNodeStyle {
    pub node_id: String,
    pub widget_type: Option<String>,
    pub bounds: Option<Bounds>,
    pub color: Option<Color>,
    pub background_color: Option<Color>,
    pub border_radius: Option<f64>,
    pub elevation: Option<f64>,
    pub text_style_label: Option<String>,
    pub text_content: Option<String>,
}

/// Parse render tree text to extract ALL style information
pub fn parse_render_tree(text: &str, scale_factor: f64) -> Vec<RenderNodeStyle> {
    let mut styles = Vec::new();

    // Regex for color that may span lines - match by components separately
    // Color format: color: Color(alpha: X, red: X, green: X, blue: X, ...)
    let re_alpha = Regex::new(r"alpha:\s*([\d.]+)").ok();
    let re_red = Regex::new(r"red:\s*([\d.]+)").ok();
    let re_green = Regex::new(r"green:\s*([\d.]+)").ok();
    let re_blue = Regex::new(r"blue:\s*([\d.]+)").ok();
    let re_debug_label = Regex::new(r"(?:labelLarge|bodyLarge|bodyMedium|bodySmall|titleLarge|titleMedium|headlineLarge|headlineMedium)").ok();
    let re_text_content = Regex::new(r#""([^"\n]{1,50})""#).ok();

    // Remove line-break formatting characters to make parsing easier
    let clean_text = text.replace("│", " ").replace("║", " ").replace("╎", " ");

    // Split by "TextSpan:" and process each section
    let sections: Vec<&str> = clean_text.split("TextSpan:").collect();
    for (i, section) in sections.iter().enumerate().skip(1) {
        // Only process the first ~800 chars of each section
        let section_part = if section.len() > 800 {
            &section[..800]
        } else {
            *section
        };

        let mut style = RenderNodeStyle {
            node_id: format!("ts{}", i),
            widget_type: Some("Text".to_string()),
            bounds: None,
            color: None,
            background_color: None,
            border_radius: None,
            elevation: None,
            text_style_label: None,
            text_content: None,
        };

        // Extract color components - they may be spread across lines
        if section_part.contains("color: Color(") {
            let alpha = re_alpha
                .as_ref()
                .and_then(|re| re.captures(section_part))
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok())
                .unwrap_or(1.0);
            let red = re_red
                .as_ref()
                .and_then(|re| re.captures(section_part))
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok());
            let green = re_green
                .as_ref()
                .and_then(|re| re.captures(section_part))
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok());
            let blue = re_blue
                .as_ref()
                .and_then(|re| re.captures(section_part))
                .and_then(|c| c.get(1))
                .and_then(|m| m.as_str().parse::<f64>().ok());

            if let (Some(r), Some(g), Some(b)) = (red, green, blue) {
                style.color = Some(Color {
                    r: (r * 255.0).round() as u8,
                    g: (g * 255.0).round() as u8,
                    b: (b * 255.0).round() as u8,
                    a: alpha,
                });
            }
        }

        // Extract debug label for font info (look for specific style names)
        if let Some(ref re) = re_debug_label {
            if let Some(label_caps) = re.captures(section_part) {
                style.text_style_label = label_caps.get(0).map(|m| m.as_str().to_string());
            }
        }

        // Extract text content (quoted strings)
        if let Some(ref re) = re_text_content {
            if let Some(text_caps) = re.captures(section_part) {
                style.text_content = text_caps.get(1).map(|m| m.as_str().to_string());
            }
        }

        if style.color.is_some() || style.text_style_label.is_some() {
            styles.push(style);
        }
    }

    // Extract background colors from RenderPhysicalShape (simpler approach)
    if let Some(re_physical) = Regex::new(r"color: Color\(alpha:\s*([\d.]+),\s*red:\s*([\d.]+),\s*green:\s*([\d.]+),\s*blue:\s*([\d.]+)").ok() {
        // Find all color definitions that might be background colors
        // We'll identify them by context (being near RenderPhysicalShape or being in color: not decoration:)
        for line in text.lines() {
            if line.contains("RenderPhysicalShape") || (line.contains("color: Color") && !line.contains("decoration:")) {
                if let Some(caps) = re_physical.captures(line) {
                    let alpha: f64 = caps.get(1).and_then(|m| m.as_str().parse().ok()).unwrap_or(1.0);
                    let red: f64 = caps.get(2).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
                    let green: f64 = caps.get(3).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);
                    let blue: f64 = caps.get(4).and_then(|m| m.as_str().parse().ok()).unwrap_or(0.0);

                    // Skip if it looks like text color (dark on light theme)
                    // Text colors typically have low RGB values
                    if red > 0.5 || green > 0.5 || blue > 0.5 {
                        styles.push(RenderNodeStyle {
                            node_id: format!("bg{}", styles.len()),
                            widget_type: Some("Container".to_string()),
                            bounds: None,
                            color: None,
                            background_color: Some(Color {
                                r: (red * 255.0).round() as u8,
                                g: (green * 255.0).round() as u8,
                                b: (blue * 255.0).round() as u8,
                                a: alpha,
                            }),
                            border_radius: None,
                            elevation: None,
                            text_style_label: None,
                            text_content: None,
                        });
                    }
                }
            }
        }
    }

    // Extract border radius from text
    if let Some(re_radius) = Regex::new(r"BorderRadius\.circular\(([\d.]+)\)").ok() {
        for caps in re_radius.captures_iter(text) {
            if let Some(radius) = caps.get(1).and_then(|m| m.as_str().parse::<f64>().ok()) {
                styles.push(RenderNodeStyle {
                    node_id: format!("clip{}", styles.len()),
                    widget_type: None,
                    bounds: None,
                    color: None,
                    background_color: None,
                    border_radius: Some(radius * scale_factor),
                    elevation: None,
                    text_style_label: None,
                    text_content: None,
                });
            }
        }
    }

    styles
}

/// Match render styles to semantics nodes by text content and element type
pub fn match_styles_to_nodes(
    nodes: &mut ElementNode,
    render_styles: &[RenderNodeStyle],
    scale_factor: f64,
) {
    // Collect styles by category
    let text_styles: Vec<&RenderNodeStyle> = render_styles
        .iter()
        .filter(|s| s.color.is_some() || s.text_style_label.is_some())
        .collect();
    let bg_styles: Vec<&RenderNodeStyle> = render_styles
        .iter()
        .filter(|s| s.background_color.is_some())
        .collect();
    let radius_styles: Vec<&RenderNodeStyle> = render_styles
        .iter()
        .filter(|s| s.border_radius.is_some())
        .collect();

    // Find default text style (usually bodyMedium for regular text)
    let default_text_style = text_styles
        .iter()
        .find(|s| {
            s.text_style_label
                .as_ref()
                .map(|l| l.contains("bodyMedium"))
                .unwrap_or(false)
        })
        .or_else(|| text_styles.first());

    // Find button text style (usually labelLarge)
    let button_text_style = text_styles.iter().find(|s| {
        s.text_style_label
            .as_ref()
            .map(|l| l.contains("labelLarge"))
            .unwrap_or(false)
    });

    match_styles_recursive(
        nodes,
        &text_styles,
        &bg_styles,
        &radius_styles,
        default_text_style.copied(),
        button_text_style.copied(),
        scale_factor,
    );
}

fn match_styles_recursive(
    node: &mut ElementNode,
    text_styles: &[&RenderNodeStyle],
    bg_styles: &[&RenderNodeStyle],
    radius_styles: &[&RenderNodeStyle],
    default_text_style: Option<&RenderNodeStyle>,
    button_text_style: Option<&RenderNodeStyle>,
    scale_factor: f64,
) {
    let mut style_info = node.style.take().unwrap_or_default();

    // Apply styles based on element type
    match node.element_type.as_str() {
        "header" | "text" => {
            // Use default text style
            if let Some(ts) = default_text_style {
                if let Some(c) = &ts.color {
                    style_info.text_color = Some(c.to_hex());
                }
                if let Some(tsl) = &ts.text_style_label {
                    apply_font_style(&mut style_info, tsl);
                }
            }
        }
        "button" => {
            // Use button text style for text color
            if let Some(ts) = button_text_style.or(default_text_style) {
                if let Some(c) = &ts.color {
                    style_info.text_color = Some(c.to_hex());
                }
                if let Some(tsl) = &ts.text_style_label {
                    apply_font_style(&mut style_info, tsl);
                }
            }
            // Use background color from physical shapes
            if let Some(bg) = bg_styles.first() {
                if let Some(c) = &bg.background_color {
                    style_info.background_color = Some(c.to_hex());
                }
            }
            // Use border radius
            if let Some(r) = radius_styles.first() {
                if let Some(br) = r.border_radius {
                    style_info.border_radius = Some(br);
                }
            }
        }
        "textField" => {
            // Text fields get default text style
            if let Some(ts) = default_text_style {
                if let Some(c) = &ts.color {
                    style_info.text_color = Some(c.to_hex());
                }
            }
        }
        _ => {}
    }

    if !style_info.is_empty() {
        node.style = Some(style_info);
    }

    // Recursively process children
    for child in &mut node.children {
        match_styles_recursive(
            child,
            text_styles,
            bg_styles,
            radius_styles,
            default_text_style,
            button_text_style,
            scale_factor,
        );
    }
}

/// Apply font style from debug label
fn apply_font_style(style_info: &mut StyleInfo, label: &str) {
    if label.contains("bodyLarge") {
        style_info.font_size = Some(16.0);
    } else if label.contains("bodyMedium") {
        style_info.font_size = Some(14.0);
    } else if label.contains("bodySmall") {
        style_info.font_size = Some(12.0);
    } else if label.contains("labelLarge") {
        style_info.font_size = Some(14.0);
        style_info.font_weight = Some("500".to_string());
    } else if label.contains("titleLarge") {
        style_info.font_size = Some(22.0);
    } else if label.contains("titleMedium") {
        style_info.font_size = Some(16.0);
        style_info.font_weight = Some("500".to_string());
    } else if label.contains("headlineLarge") {
        style_info.font_size = Some(32.0);
    } else if label.contains("headlineMedium") {
        style_info.font_size = Some(28.0);
    }
}
