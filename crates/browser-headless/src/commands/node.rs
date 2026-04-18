use std::fmt::Write as _;

use html_dom::{DocumentRoot, DomNode, NodeData, NodeId};

use crate::{HeadlessEngine, commands::layout::print_layout_node};

pub fn cmd_node_id(engine: &HeadlessEngine, id: usize) -> Result<(), String> {
    let Some(page) = &engine.page else {
        return Err("No page loaded. Please navigate to a URL first.".to_string());
    };

    let document = page.document();
    let node_id = NodeId(id);
    let node = document
        .get_node(&node_id)
        .ok_or_else(|| format!("Node {} not found in DOM", id))?;

    println!("Node {}", id);
    println!("Type: {}", describe_node_type(node));

    match node.parent {
        Some(parent_id) => println!("Parent: {}", parent_id.0),
        None => println!("Parent: none (root node)"),
    }

    if node.children.is_empty() {
        println!("Children: none");
    } else {
        let child_ids = node
            .children
            .iter()
            .map(|child_id| child_id.0.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("Children ({}): {}", node.children.len(), child_ids);
    }

    Ok(())
}

pub fn cmd_node_dom(engine: &HeadlessEngine, id: usize, max_depth: Option<usize>) -> Result<(), String> {
    let Some(page) = &engine.page else {
        return Err("No page loaded. Please navigate to a URL first.".to_string());
    };

    let document = page.document();
    let node_id = NodeId(id);
    document
        .get_node(&node_id)
        .ok_or_else(|| format!("Node {} not found in DOM", id))?;

    let mut output = String::new();
    write_dom_subtree(document, node_id, 0, max_depth, &mut output).map_err(|e| e.to_string())?;

    if output.trim().is_empty() {
        println!("Node {} has no printable DOM output", id);
    } else {
        print!("{output}");
    }

    Ok(())
}

pub fn cmd_node_style(engine: &mut HeadlessEngine, id: usize) -> Result<(), String> {
    engine.ensure_layout()?;

    let node_id = NodeId(id);

    let Some(style_tree) = engine.style_tree.as_ref() else {
        return Err("Style tree not available".to_string());
    };

    let styled_node = style_tree
        .find_node(&node_id)
        .ok_or_else(|| format!("Node {} not found in style tree", id))?;

    println!("Computed style for node {}:", id);
    println!("{:#?}", styled_node.style);

    Ok(())
}

pub fn cmd_node_layout(engine: &mut HeadlessEngine, id: usize) -> Result<(), String> {
    engine.ensure_layout()?;

    let node_id = NodeId(id);
    let Some(layout) = engine.layout_tree.as_ref() else {
        return Err("Layout not available".to_string());
    };

    let path = layout.find_path(node_id).ok_or_else(|| {
        format!("Node {} is not present in the layout tree (it may not render, e.g. display:none)", id)
    })?;
    let node = layout
        .node_at(&path)
        .ok_or_else(|| format!("Layout node {} could not be resolved by path", id))?;

    println!("Layout subtree for node {}:", id);
    print_layout_node(node, 0);

    Ok(())
}

pub fn cmd_node_children(engine: &HeadlessEngine, id: usize, recursive: bool) -> Result<(), String> {
    let Some(page) = &engine.page else {
        return Err("No page loaded. Please navigate to a URL first.".to_string());
    };

    let document = page.document();
    let node_id = NodeId(id);
    let node = document
        .get_node(&node_id)
        .ok_or_else(|| format!("Node {} not found in DOM", id))?;

    if node.children.is_empty() {
        println!("Node {} has no children", id);
        return Ok(());
    }

    if recursive {
        println!("Descendants of node {}:", id);
        for child_id in &node.children {
            print_descendants(document, *child_id, 1)?;
        }
    } else {
        println!("Children of node {}:", id);
        for child_id in &node.children {
            let child = document
                .get_node(child_id)
                .ok_or_else(|| format!("Node {} references missing child {}", id, child_id.0))?;
            println!("  [{}] {}", child_id.0, describe_node_type(child));
        }
    }

    Ok(())
}

fn print_descendants(document: &DocumentRoot, node_id: NodeId, depth: usize) -> Result<(), String> {
    let node = document
        .get_node(&node_id)
        .ok_or_else(|| format!("DOM references missing descendant node {}", node_id.0))?;

    let indent = "  ".repeat(depth);
    println!("{}[{}] {}", indent, node_id.0, describe_node_type(node));

    for child_id in &node.children {
        print_descendants(document, *child_id, depth + 1)?;
    }

    Ok(())
}

fn describe_node_type(node: &DomNode) -> String {
    match &node.data {
        NodeData::Element(element) => format!("Element <{}>", element.tag_name()),
        NodeData::Text(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                "Text (whitespace)".to_string()
            } else {
                format!("Text \"{}\"", trimmed)
            }
        }
    }
}

fn write_dom_subtree(
    document: &DocumentRoot,
    node_id: NodeId,
    depth: usize,
    max_depth: Option<usize>,
    output: &mut String,
) -> std::fmt::Result {
    let Some(node) = document.get_node(&node_id) else {
        return Ok(());
    };

    let indent = "  ".repeat(depth);

    match &node.data {
        NodeData::Element(element) => {
            write!(output, "{}<{} data-node-id=\"{}\"", indent, element.tag_name(), node.id.0)?;
            for (name, value) in &element.attributes {
                if name.trim().is_empty() {
                    continue;
                }
                write!(output, " {}=\"{}\"", name, value)?;
            }
            writeln!(output, ">")?;

            let can_descend = max_depth.is_none_or(|max| depth < max);
            if can_descend {
                for child_id in &node.children {
                    write_dom_subtree(document, *child_id, depth + 1, max_depth, output)?;
                }
            } else if !node.children.is_empty() {
                writeln!(output, "{}  ...", indent)?;
            }

            if !element.tag.is_void_element() {
                writeln!(output, "{}</{}>", indent, element.tag_name())?;
            }
        }
        NodeData::Text(text) => {
            if !text.trim().is_empty() {
                writeln!(output, "{}{}", indent, text.trim())?;
            }
        }
    }

    Ok(())
}
