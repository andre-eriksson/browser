use layout::LayoutNode;
use tracing::info;

use crate::HeadlessEngine;

pub fn cmd_node(engine: &mut HeadlessEngine, x: f32, y: f32) -> Result<(), String> {
    engine.ensure_layout()?;

    let Some(ref layout) = engine.layout_tree else {
        return Err("Layout not available".to_string());
    };

    let nodes = layout.resolve(x, y);

    if nodes.is_empty() {
        println!("No node at ({}, {})", x, y);
    } else {
        println!("Nodes at ({}, {}):", x, y);
        for node in nodes {
            print_layout_node(node, 1);
        }
    }

    Ok(())
}

pub fn cmd_layout(engine: &mut HeadlessEngine) -> Result<(), String> {
    engine.ensure_layout()?;

    let Some(ref layout) = engine.layout_tree else {
        return Err("Layout not available".to_string());
    };

    println!("Layout Tree ({}x{}):", layout.content_width, layout.content_height);
    for node in &layout.root_nodes {
        print_layout_node(node, 0);
    }

    Ok(())
}

pub fn cmd_resize(engine: &mut HeadlessEngine, width: f32, height: f32) -> Result<(), String> {
    if width <= 0.0 || height <= 0.0 {
        return Err("Viewport dimensions must be positive".to_string());
    }

    engine.viewport_width = width;
    engine.viewport_height = height;
    engine.recompute_layout();
    info!("Viewport resized to {}x{}", width, height);
    Ok(())
}

pub fn print_layout_node(node: &LayoutNode, depth: usize) {
    let indent = "  ".repeat(depth);
    let rect = &node.dimensions;
    println!(
        "{}[{}] x={:.1} y={:.1} w={:.1} h={:.1}",
        indent, node.node_id.0, rect.x, rect.y, rect.width, rect.height
    );

    for child in &node.children {
        print_layout_node(child, depth + 1);
    }
}
