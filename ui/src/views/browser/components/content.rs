use std::sync::{Arc, Mutex};

use iced::{
    Background, Color, Length,
    widget::{Column, container},
};

use api::dom::ConcurrentDomNode;

use crate::{api::message::Message, core::app::Application, renderer::html::display_html};

/// Renders the content of the browser window, displaying HTML content from the current tab.
pub fn render_content<'window>(
    app: &'window Application,
) -> Result<container::Container<'window, Message>, String> {
    let html_content = if let Ok(html) = app.tabs[app.current_tab_id].html_content.lock() {
        html.clone()
    } else {
        return Err("Failed to lock HTML content".to_string());
    };

    match html_content {
        ConcurrentDomNode::Document(children) => {
            if children.is_empty() {
                return Err("No HTML content loaded - document is empty.".to_string());
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
                None => Err("No body element found in HTML document.".to_string()),
            }
        }
        _ => Err("HTML content is not a document node.".to_string()),
    }
}

fn display_child_elements<'window>(
    children: Vec<Arc<Mutex<ConcurrentDomNode>>>,
) -> Option<Column<'window, Message>> {
    for child in children {
        if let ConcurrentDomNode::Element(element) = child.lock().unwrap().clone() {
            if element.tag_name == "body" {
                return Some(display_html(element));
            }

            if let Some(result) = display_child_elements(element.children) {
                return Some(result);
            }
        }
    }

    None
}
