use std::sync::{Arc, Mutex};

use iced::{
    Color,
    widget::{Column, column, horizontal_rule, row, text},
};

use api::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{
    api::message::Message,
    renderer::{
        layout::{ElementType, get_element_type},
        util::{get_margin_for_element, get_text_style_for_element},
    },
};

/// Main entry point for rendering HTML content using Iced widgets.
///
/// This function takes a ConcurrentElement (which should be a body tag) and converts it
/// into an Iced Column widget that can be displayed in the UI.
///
/// # Arguments
/// * `element` - The root HTML element to render, typically a `<body>` tag
///
/// # Returns
/// * `Column<'window, Message>` - An Iced column widget containing the rendered HTML content
pub fn display_html<'window>(element: ConcurrentElement) -> Column<'window, Message> {
    if element.tag_name != "body" {
        return column![text("Only 'body' tag is supported for rendering.").color(Color::BLACK)];
    }

    column![process_dom_children(&element.children)]
}

/// Processes a list of DOM nodes and converts them into Iced elements.
/// This is the main recursive function that handles the DOM tree traversal.
///
/// # Arguments
/// * `nodes` - A slice of DOM nodes to process
///
/// # Returns
/// * `iced::Element<'window, Message>` - An Iced element representing the rendered nodes
fn process_dom_children<'window>(
    nodes: &[Arc<Mutex<ConcurrentDomNode>>],
) -> iced::Element<'window, Message> {
    process_dom_children_with_context(nodes, None)
}

/// Processes DOM nodes with additional context about the parent element.
/// This allows for better styling and layout decisions based on the parent context.
///
/// # Arguments
/// * `nodes` - A slice of DOM nodes to process
/// * `parent_element` - Optional reference to the parent element for context
///
/// # Returns
/// * `iced::Element<'window, Message>` - An Iced element representing the rendered nodes
fn process_dom_children_with_context<'window>(
    nodes: &[Arc<Mutex<ConcurrentDomNode>>],
    parent_element: Option<&ConcurrentElement>,
) -> iced::Element<'window, Message> {
    let mut elements = Vec::new();

    for node_arc in nodes {
        let node = node_arc.lock().unwrap().clone();

        match node {
            ConcurrentDomNode::Element(element) => {
                if &element.tag_name == "hr" {
                    elements.push(horizontal_rule(1).into());
                    continue;
                }

                match get_element_type(&element.tag_name) {
                    ElementType::Block => {
                        let is_preformatted = &element.tag_name == "pre";

                        let children_content = if is_preformatted {
                            process_preformatted_content(&element.children, &element)
                        } else {
                            process_dom_children_with_context(&element.children, Some(&element))
                        };

                        let margin = get_margin_for_element(&element.tag_name);
                        if margin > 0 && !elements.is_empty() {
                            elements.push(text("").size(margin / 2).into());
                        }

                        elements.push(children_content);
                    }
                    ElementType::Inline => {
                        let inline_content =
                            process_dom_children_with_context(&element.children, Some(&element));
                        elements.push(inline_content);
                    }
                    ElementType::ListItem => {
                        let bullet = text(match element.tag_name.as_str() {
                            "li" => " • ",
                            "dt" => " • ",
                            "summary" => " ▶ ",
                            _ => "",
                        })
                        .color(Color::BLACK);

                        let content =
                            process_dom_children_with_context(&element.children, Some(&element));
                        let list_item = row![bullet, content];
                        elements.push(list_item.into());
                    }
                    ElementType::Skip => {
                        continue;
                    }
                    ElementType::Unknown => {
                        let children_content =
                            process_dom_children_with_context(&element.children, Some(&element));
                        elements.push(children_content);
                    }
                }
            }
            ConcurrentDomNode::Text(content) => {
                if !content.trim().is_empty() {
                    let styled_text = if let Some(parent) = parent_element {
                        get_text_style_for_element(&parent.tag_name, content.clone())
                    } else {
                        text(content).color(Color::BLACK)
                    };
                    elements.push(styled_text.into());
                }
            }
            _ => {
                elements.push(
                    text("Unsupported node type")
                        .color(Color::from_rgb(1.0, 0.5, 0.0))
                        .into(),
                );
            }
        }
    }

    if elements.is_empty() {
        text("").into()
    } else if elements.len() == 1 {
        elements.into_iter().next().unwrap()
    } else {
        column(elements).into()
    }
}

/// Handles preformatted content (like <pre> tags) which should preserve whitespace and line breaks
fn process_preformatted_content<'window>(
    nodes: &[Arc<Mutex<ConcurrentDomNode>>],
    parent_element: &ConcurrentElement,
) -> iced::Element<'window, Message> {
    let mut elements = Vec::new();

    for node_arc in nodes {
        let node = node_arc.lock().unwrap().clone();

        match node {
            ConcurrentDomNode::Text(content) => {
                for line in content.split('\n') {
                    if !line.is_empty() {
                        let styled_text =
                            get_text_style_for_element(&parent_element.tag_name, line.to_string());
                        elements.push(styled_text.into());
                    } else {
                        elements.push(text("").size(8).into());
                    }
                }
            }
            ConcurrentDomNode::Element(element) => {
                let nested_content =
                    process_dom_children_with_context(&element.children, Some(&element));
                elements.push(nested_content);
            }
            _ => {}
        }
    }

    if elements.is_empty() {
        text("").into()
    } else {
        column(elements).into()
    }
}
