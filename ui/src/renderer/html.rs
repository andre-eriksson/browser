use std::sync::{Arc, Mutex};

use iced::{
    Color,
    widget::{Column, column, horizontal_rule, row, text},
};

use html_parser::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{
    api::message::Message,
    renderer::{
        inline::compose_inline_elements,
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

/// Processes mixed content (text nodes and inline elements) within a block element.
/// This preserves the natural flow of content where text and inline elements are intermixed.
///
/// # Arguments
/// * `nodes` - A slice of DOM nodes to process
/// * `parent_element` - The parent block element containing the mixed content
///
/// # Returns
/// * `iced::Element<'window, Message>` - An Iced element representing the mixed content as a row
fn process_mixed_content<'window>(
    nodes: &[Arc<Mutex<ConcurrentDomNode>>],
    parent_element: &ConcurrentElement,
) -> iced::Element<'window, Message> {
    let mut inline_elements = Vec::new();

    for node_arc in nodes {
        let node = node_arc.lock().unwrap().clone();

        match node {
            ConcurrentDomNode::Element(element) => {
                if get_element_type(&element.tag_name) == ElementType::Inline {
                    for child_arc in &element.children {
                        let child = child_arc.lock().unwrap().clone();
                        if let ConcurrentDomNode::Text(content) = child {
                            let styled_text =
                                get_text_style_for_element(&element.tag_name, content);
                            inline_elements.push(styled_text.into());
                        }
                    }
                } else {
                    // Handle block elements within mixed content (shouldn't happen in well-formed HTML)
                    let children_content =
                        process_dom_children_with_context(&element.children, Some(&element));
                    inline_elements.push(children_content);
                }
            }
            ConcurrentDomNode::Text(content) => {
                let styled_text = get_text_style_for_element(&parent_element.tag_name, content);
                inline_elements.push(styled_text.into());
            }
            _ => {}
        }
    }

    if inline_elements.is_empty() {
        text("").into()
    } else {
        row(inline_elements).into()
    }
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
    let mut inline_buffer = Vec::new();

    for node_arc in nodes {
        let node = node_arc.lock().unwrap().clone();

        match node {
            ConcurrentDomNode::Element(element) => {
                if &element.tag_name == "hr" {
                    elements.push(horizontal_rule(1).into());
                    continue;
                }

                let has_text_nodes = element.children.iter().any(|child| {
                    matches!(child.lock().unwrap().clone(), ConcurrentDomNode::Text(_))
                });

                let has_inline_elements = element.children.iter().any(|child| {
                    if let ConcurrentDomNode::Element(inner_element) = child.lock().unwrap().clone()
                    {
                        get_element_type(&inner_element.tag_name) == ElementType::Inline
                    } else {
                        false
                    }
                });

                let is_mixed_content = has_text_nodes && has_inline_elements;

                match get_element_type(&element.tag_name) {
                    ElementType::Block => {
                        // Clear any pending inline elements before processing block
                        if !inline_buffer.is_empty() {
                            if let Some(el) =
                                compose_inline_elements(&inline_buffer, parent_element, None)
                            {
                                elements.push(el);
                            }
                            inline_buffer.clear();
                        }

                        // If this block element has mixed content, handle it specially
                        if is_mixed_content {
                            let inline_content = process_mixed_content(&element.children, &element);

                            let margin = get_margin_for_element(&element.tag_name);
                            if margin > 0 && !elements.is_empty() {
                                elements.push(text("").size(margin / 2).into());
                            }

                            elements.push(inline_content);
                        } else {
                            let children_content = process_dom_children_with_context(
                                &element.children,
                                Some(&element),
                            );

                            let margin = get_margin_for_element(&element.tag_name);
                            if margin > 0 && !elements.is_empty() {
                                elements.push(text("").size(margin / 2).into());
                            }

                            elements.push(row![children_content].into());
                        }
                    }
                    ElementType::Inline => {
                        inline_buffer.push(element.clone());
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
                let styled_text = if let Some(parent) = parent_element {
                    get_text_style_for_element(&parent.tag_name, content.clone())
                } else {
                    text(content).color(Color::BLACK)
                };

                elements.push(styled_text.into());
            }
            _ => {}
        }
    }

    compose_inline_elements(&inline_buffer, parent_element, None)
        .into_iter()
        .for_each(|el| {
            elements.push(el);
        });

    if elements.is_empty() {
        text("").into()
    } else if elements.len() == 1 {
        elements.into_iter().next().unwrap()
    } else {
        column(elements).into()
    }
}
