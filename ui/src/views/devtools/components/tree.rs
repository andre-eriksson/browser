use std::sync::{Arc, Mutex};

use api::dom::{ConcurrentDomNode, ConcurrentElement};
use iced::{
    Background, Color, Font, Length,
    widget::{Column, column, container, row, text},
};

use crate::{api::message::Message, core::app::Application};

/// Renders the content of the browser window, displaying HTML content from the current tab.
pub fn render_dom_tree(app: &Application) -> Result<container::Container<'_, Message>, String> {
    let html_content = if let Ok(html) = app.tabs[app.current_tab_id].html_content.lock() {
        html.clone()
    } else {
        return Err("Failed to lock DOM tree".to_string());
    };

    match html_content {
        ConcurrentDomNode::Document(children) => {
            if children.is_empty() {
                return Err("No DOM content loaded - document is empty.".to_string());
            }

            match display_child_elements(children) {
                Some(content) => {
                    let content: container::Container<'_, Message> = container(content)
                        .style(|_theme| {
                            container::background(Background::Color(Color::from_rgb(
                                0.95, 0.95, 0.95,
                            )))
                        })
                        .padding(10.0)
                        .width(Length::Fill);
                    Ok(content)
                }
                None => Err("No body element found in DOM document.".to_string()),
            }
        }
        _ => Err("DOM content is not a document node.".to_string()),
    }
}

fn display_child_elements<'window>(
    children: Vec<Arc<Mutex<ConcurrentDomNode>>>,
) -> Option<Column<'window, Message>> {
    for child in children {
        if let ConcurrentDomNode::Element(element) = child.lock().unwrap().clone() {
            return Some(display_dom_tree(element));
        }
    }

    None
}

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
pub fn display_dom_tree<'window>(element: ConcurrentElement) -> Column<'window, Message> {
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
    process_dom_children_with_context(nodes, 0)
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
    depth: usize,
) -> iced::Element<'window, Message> {
    let mut elements = Vec::new();

    for node_arc in nodes {
        let node = node_arc.lock().unwrap().clone();
        let spacing = " ".repeat(depth * 2);

        match node {
            ConcurrentDomNode::Element(element) => {
                let is_void_element = matches!(
                    element.tag_name.as_str(),
                    "area"
                        | "base"
                        | "br"
                        | "col"
                        | "embed"
                        | "hr"
                        | "img"
                        | "input"
                        | "link"
                        | "meta"
                        | "param"
                        | "source"
                        | "track"
                        | "wbr"
                );

                // Create opening tag with highlighting
                let opening_tag = if element.attributes.is_empty() {
                    row![
                        text(format!("{}<", spacing))
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                            }),
                        text(element.clone().tag_name)
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.0, 0.4, 0.8))
                            }),
                        text(if is_void_element { " />" } else { ">" })
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                            }),
                    ]
                    .into()
                } else {
                    let attrs: Vec<String> = element
                        .attributes
                        .iter()
                        .map(|(key, value)| format!("{}=\"{}\"", key, value))
                        .collect();
                    row![
                        text(format!("{}<", spacing))
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                            }),
                        text(element.clone().tag_name)
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.0, 0.4, 0.8))
                            }),
                        text(format!(" {}", attrs.join(" ")))
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.6, 0.3, 0.0))
                            }),
                        text(if is_void_element { " />" } else { ">" })
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                            }),
                    ]
                    .into()
                };

                elements.push(opening_tag);

                if !is_void_element {
                    elements.push(process_dom_children_with_context(
                        &element.children,
                        depth + 1,
                    ));

                    // Create closing tag with highlighting
                    let closing_tag = row![
                        text(format!("{}</", spacing))
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                            }),
                        text(element.clone().tag_name)
                            .font(Font::MONOSPACE)
                            .style(|_theme| text::Style {
                                color: Some(Color::from_rgb(0.0, 0.4, 0.8))
                            }),
                        text(">").font(Font::MONOSPACE).style(|_theme| text::Style {
                            color: Some(Color::from_rgb(0.5, 0.5, 0.5))
                        }),
                    ]
                    .into();

                    elements.push(closing_tag);
                }
            }
            ConcurrentDomNode::Text(content) => elements.push(
                text(format!("{}{}", spacing, content))
                    .font(Font::MONOSPACE)
                    .into(),
            ),
            _ => {}
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
