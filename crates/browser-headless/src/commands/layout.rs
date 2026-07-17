use css_display::LayoutNodeId;
use layout::LayoutTree;
use tracing::info;

use crate::HeadlessEngine;

pub fn cmd_node(engine: &mut HeadlessEngine, x: f64, y: f64) -> Result<(), String> {
    engine.ensure_layout()?;

    let Some(ref layout) = engine.layout_tree else {
        return Err("Layout not available".to_string());
    };

    let nodes = layout.resolve(x, y);

    if nodes.is_empty() {
        println!("No node at ({x}, {y})");
    } else {
        println!("Nodes at ({x}, {y}):");
        for node in nodes {
            print_layout_node(layout, &node.layout_id, 1);
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

    for node_id in &layout.root_nodes {
        print_layout_node(layout, node_id, 0);
    }

    Ok(())
}

pub fn cmd_resize(engine: &mut HeadlessEngine, width: f64, height: f64) -> Result<(), String> {
    if width <= 0.0 || height <= 0.0 {
        return Err("Viewport dimensions must be positive".to_string());
    }

    engine.viewport_width = width;
    engine.viewport_height = height;
    engine.recompute_layout();
    info!("Viewport resized to {}x{}", width, height);
    Ok(())
}

pub fn print_layout_node(layout_tree: &LayoutTree, node_id: &LayoutNodeId, depth: usize) {
    let indent = "  ".repeat(depth);

    let Some(Some(node)) = &layout_tree.nodes.get(node_id.index()) else {
        println!("{}[{:?}] (node not found)", indent, node_id);
        return;
    };

    let rect = &node.dimensions;

    if let Some(node_id) = node.node_id {
        print!("{}[{:?}] (node_id: {}) ", indent, node.layout_id, node_id);
    } else {
        print!("{}[{:?}] ", indent, node.layout_id);
    }

    println!("x={:.1} y={:.1} w={:.1} h={:.1}", rect.x, rect.y, rect.width, rect.height);

    for child in &node.children {
        print_layout_node(layout_tree, child, depth + 1);
    }
}
