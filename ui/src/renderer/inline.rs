use iced::{
    Font,
    widget::{column, row, text},
};

use api::dom::{ConcurrentDomNode, ConcurrentElement};

use crate::{api::message::Message, renderer::util::get_text_style_for_element};

/// Composes inline elements into a single Iced Element.
///
/// # Arguments
/// * `inline_buffer` - A vector of inline elements to compose.
/// * `parent_element` - An optional parent element that may influence the rendering.
/// * `additional_element` - An optional additional element to include in the composition.
///
/// # Returns
/// * `Option<iced::Element<'html, Message>>` - An Iced Element containing the composed inline elements, or None if no elements are provided.
pub fn compose_inline_elements<'html>(
    inline_buffer: &Vec<ConcurrentElement>,
    parent_element: Option<&ConcurrentElement>,
    additional_element: Option<&ConcurrentElement>,
) -> Option<iced::Element<'html, Message>> {
    let mut all_elements = inline_buffer.clone();
    if let Some(additional) = additional_element {
        all_elements.push(additional.clone());
    }
    if all_elements.is_empty() {
        return None;
    }

    let processed_element_buffer;

    if let Some(parent) = parent_element {
        if parent.tag_name == "pre" && all_elements.iter().any(|e| e.tag_name == "code") {
            processed_element_buffer = Some({
                let mut elements = Vec::new();
                for el in all_elements {
                    elements.extend(render_element(Some(parent), &el));
                }

                column(elements).into()
            });
        } else {
            let mut elements = Vec::new();
            for el in all_elements {
                elements.extend(render_element(Some(parent), &el));
            }

            processed_element_buffer = Some(row(elements).into());
        }
    } else {
        let mut elements = Vec::new();
        for el in all_elements {
            elements.extend(render_element(None, &el));
        }

        processed_element_buffer = Some(row(elements).into());
    }

    processed_element_buffer
}

fn render_element<'html>(
    _parent_element: Option<&ConcurrentElement>,
    element: &ConcurrentElement,
) -> Vec<iced::Element<'html, Message>> {
    let mut elements = Vec::new();
    for child in &element.children {
        match child.lock().unwrap().clone() {
            ConcurrentDomNode::Text(content) => match element.tag_name.as_str() {
                "code" => {
                    let formatted_content = content.replace("\r\n", "").replace('\n', "");
                    elements.push(text(formatted_content).font(Font::MONOSPACE).into());
                }
                _ => {
                    let styled_text = get_text_style_for_element(&element.tag_name, content);
                    elements.push(styled_text.into());
                }
            },
            _ => {}
        }
    }

    elements
}
