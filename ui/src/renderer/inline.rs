use std::sync::{Arc, RwLock};

use api::html::{HtmlTag, KnownTag};
use iced::{
    Background, Color, Padding,
    widget::{button, column, row, text},
};

use html_parser::dom::{DocumentNode, Element, MultiThreaded};

use crate::{
    api::message::Message, renderer::util::get_text_style_for_element, util::font::MONOSPACE,
};

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
    inline_buffer: &Vec<Arc<RwLock<DocumentNode<MultiThreaded>>>>,
    parent_element: Option<&Element<MultiThreaded>>,
    additional_element: Option<&Element<MultiThreaded>>,
) -> Option<iced::Element<'html, Message>> {
    let mut all_elements = inline_buffer.to_owned();

    if let Some(additional) = additional_element {
        all_elements.push(Arc::new(RwLock::new(DocumentNode::Element(
            additional.clone(),
        ))));
    }
    if all_elements.is_empty() {
        return None;
    }

    let processed_element_buffer;

    if let Some(parent) = parent_element {
        if parent.tag == HtmlTag::Known(KnownTag::Pre)
            && all_elements.iter().any(|e| {
                let element = e.read().unwrap().clone();

                if let Some(el) = element.as_element() {
                    el.tag == HtmlTag::Known(KnownTag::Code)
                } else {
                    false
                }
            })
        {
            processed_element_buffer = Some({
                let mut elements = Vec::new();
                for node in all_elements {
                    let element = node.read().unwrap().clone();

                    if let Some(el) = element.as_element() {
                        elements.extend(render_element(Some(parent), el));
                    }
                }

                column(elements).into()
            });
        } else {
            let mut elements = Vec::new();
            for node in all_elements {
                let element = node.read().unwrap().clone();

                if let Some(el) = element.as_element() {
                    elements.extend(render_element(Some(parent), el));
                }
            }

            processed_element_buffer = Some(row(elements).into());
        }
    } else {
        let mut elements = Vec::new();
        for node in all_elements {
            let element = node.read().unwrap().clone();

            if let Some(el) = element.as_element() {
                elements.extend(render_element(None, el));
            }
        }

        processed_element_buffer = Some(row(elements).into());
    }

    processed_element_buffer
}

fn render_element<'html>(
    _parent_element: Option<&Element<MultiThreaded>>,
    element: &Element<MultiThreaded>,
) -> Vec<iced::Element<'html, Message>> {
    let mut elements = Vec::new();
    for child in &element.children {
        if let DocumentNode::Text(content) = child.read().unwrap().clone() {
            match element.tag {
                HtmlTag::Known(KnownTag::Code) => {
                    let formatted_content = content.replace("\r\n", "").replace('\n', "");
                    elements.push(text(formatted_content).font(MONOSPACE).into());
                }
                HtmlTag::Known(KnownTag::A) => {
                    let url = element.attributes.get("href").cloned().unwrap_or_default();
                    let link_text = button(text(content).color(Color::from_rgb(0.0, 0.0, 1.0)))
                        .style(|_, _| button::Style {
                            background: Some(Background::Color(Color::from_rgb(1.0, 1.0, 1.0))),
                            ..Default::default()
                        })
                        .padding(Padding::ZERO)
                        .on_press(Message::NavigateTo(url));
                    elements.push(link_text.into());
                }
                _ => {
                    let styled_text = get_text_style_for_element(&element.tag, content);
                    elements.push(styled_text.into());
                }
            }
        }
    }

    elements
}
