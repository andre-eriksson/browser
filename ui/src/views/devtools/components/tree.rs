use std::sync::{Arc, RwLock};

use iced::{
    Background, Color, Length,
    widget::{Column, column, container, text},
};

use html_parser::dom::{DocumentNode, Element, MultiThreaded};

use crate::{api::message::Message, core::app::Application, util::font::MONOSPACE};

const TAG_STYLE: fn(&iced::Theme) -> iced::widget::text::Style =
    |_theme| iced::widget::text::Style {
        color: Some(Color::from_rgb(0.0, 0.4, 0.8)),
    };

const MAX_RENDER_DEPTH: usize = 10;
const MAX_CHILDREN_LIMIT: usize = 100;

/// Renders the content of the browser window, displaying HTML content from the current tab.
pub fn render_dom_tree(app: &Application) -> Result<container::Container<'_, Message>, String> {
    let root = &app.tabs[app.current_tab_id].html_content;

    if root.nodes.is_empty() {
        return Err("No DOM content loaded - document is empty.".to_string());
    }

    match display_child_elements(&root.nodes) {
        Some(content) => {
            let content: container::Container<'_, Message> = container(content)
                .style(|_theme| {
                    container::background(Background::Color(Color::from_rgb(0.95, 0.95, 0.95)))
                })
                .padding(10.0)
                .width(Length::Fill);
            Ok(content)
        }
        None => Err("No body element found in DOM document.".to_string()),
    }
}

fn display_child_elements<'window>(
    children: &[Arc<RwLock<DocumentNode<MultiThreaded>>>],
) -> Option<Column<'window, Message>> {
    for child in children {
        if let DocumentNode::Element(element) = child.read().unwrap().clone() {
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
pub fn display_dom_tree<'window>(element: Element<MultiThreaded>) -> Column<'window, Message> {
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
    nodes: &[Arc<RwLock<DocumentNode<MultiThreaded>>>],
) -> iced::Element<'window, Message> {
    process_dom_children_with_context(nodes, 0)
}

/// Processes DOM nodes with additional context about the parent element.
/// This allows for better styling and layout decisions based on the parent context.
///
/// # Arguments
/// * `nodes` - A slice of DOM nodes to process
/// * `depth` - Current nesting depth for indentation
///
/// # Returns
/// * `iced::Element<'window, Message>` - An Iced element representing the rendered nodes
fn process_dom_children_with_context<'window>(
    nodes: &[Arc<RwLock<DocumentNode<MultiThreaded>>>],
    depth: usize,
) -> iced::Element<'window, Message> {
    // Prevent excessive nesting that can cause performance issues
    if depth > MAX_RENDER_DEPTH {
        return text(format!(
            "{}... (content truncated at depth {})",
            " ".repeat(depth * 2),
            MAX_RENDER_DEPTH
        ))
        .font(MONOSPACE)
        .style(|_theme| iced::widget::text::Style {
            color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
        })
        .into();
    }

    let mut elements = Vec::new();

    // Limit the number of children to prevent performance issues with massive DOMs
    let nodes_to_process = if nodes.len() > MAX_CHILDREN_LIMIT {
        &nodes[..MAX_CHILDREN_LIMIT]
    } else {
        nodes
    };

    let spacing = " ".repeat(depth * 2);

    for node_arc in nodes_to_process {
        let node = node_arc.read().unwrap().clone();

        match node {
            DocumentNode::Element(element) => {
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

                // Create a single text widget for the entire opening tag to reduce widget count
                let opening_tag_text = if element.attributes.is_empty() {
                    format!(
                        "{}<{}{}",
                        spacing,
                        element.tag_name,
                        if is_void_element { " />" } else { ">" }
                    )
                } else {
                    let attrs: Vec<String> = element
                        .attributes
                        .iter()
                        .take(5) // Limit attributes shown for performance
                        .map(|(key, value)| {
                            // Truncate long attribute values
                            let truncated_value = if value.len() > 50 {
                                format!("{}...", &value[..47])
                            } else {
                                value.clone()
                            };
                            format!("{}=\"{}\"", key, truncated_value)
                        })
                        .collect();

                    let attr_suffix = if element.attributes.len() > 5 {
                        " ..."
                    } else {
                        ""
                    };

                    format!(
                        "{}<{} {}{}{}",
                        spacing,
                        element.tag_name,
                        attrs.join(" "),
                        attr_suffix,
                        if is_void_element { " />" } else { ">" }
                    )
                };

                elements.push(
                    text(opening_tag_text)
                        .font(MONOSPACE)
                        .style(TAG_STYLE)
                        .into(),
                );

                if !is_void_element {
                    // Only recurse if we have a reasonable number of children
                    if element.children.len() <= MAX_CHILDREN_LIMIT {
                        elements.push(process_dom_children_with_context(
                            &element.children,
                            depth + 1,
                        ));
                    } else {
                        elements.push(
                            text(format!(
                                "{}  ... ({} children, showing first {})",
                                spacing,
                                element.children.len(),
                                MAX_CHILDREN_LIMIT
                            ))
                            .font(MONOSPACE)
                            .style(|_theme| iced::widget::text::Style {
                                color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
                            })
                            .into(),
                        );

                        elements.push(process_dom_children_with_context(
                            &element.children[..MAX_CHILDREN_LIMIT.min(element.children.len())],
                            depth + 1,
                        ));
                    }

                    // Create closing tag
                    elements.push(
                        text(format!("{}</{}>", spacing, element.tag_name))
                            .font(MONOSPACE)
                            .style(TAG_STYLE)
                            .into(),
                    );
                }
            }
            DocumentNode::Text(content) => {
                // Truncate very long text content
                let truncated_content = if content.len() > 200 {
                    format!("{}{}... (truncated)", spacing, &content[..197])
                } else {
                    format!("{}{}", spacing, content)
                };

                elements.push(text(truncated_content).font(MONOSPACE).into());
            }
        }
    }

    // Show truncation message if we limited the nodes
    if nodes.len() > MAX_CHILDREN_LIMIT {
        elements.push(
            text(format!(
                "{}... ({} more children not shown)",
                spacing,
                nodes.len() - MAX_CHILDREN_LIMIT
            ))
            .font(MONOSPACE)
            .style(|_theme| iced::widget::text::Style {
                color: Some(Color::from_rgb(0.7, 0.7, 0.7)),
            })
            .into(),
        );
    }

    if elements.is_empty() {
        text("").into()
    } else if elements.len() == 1 {
        elements.into_iter().next().unwrap()
    } else {
        column(elements).spacing(1).into()
    }
}
