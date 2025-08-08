use api::html::{HtmlTag, KnownTag};
use iced::{
    Background, Color, Length,
    widget::{container, text},
};

use crate::{api::message::Message, core::app::Application, renderer::html::display_html};

/// Renders the content of the browser window, displaying HTML content from the current tab.
pub fn render_content<'window>(
    app: &'window Application,
) -> container::Container<'window, Message> {
    let root = &app.tabs[app.current_tab_id].html_content;

    if root.nodes.is_empty() {
        return render_blank();
    }

    if let Some(body_node_guard) = root
        .index
        .first_element_by_tag(&HtmlTag::Known(KnownTag::Body))
    {
        if let Some(body_element) = body_node_guard.as_element() {
            return container(display_html(body_element))
                .style(|_theme| {
                    container::background(Background::Color(Color::from_rgb(0.95, 0.95, 0.95)))
                })
                .padding(10.0)
                .width(Length::Fill);
        }
    }

    // Fallback if no body element found
    render_error()
}

/// Renders a blank page.
fn render_blank<'window>() -> container::Container<'window, Message> {
    container(text("About: Blank"))
        .width(Length::Fill)
        .padding(10.0)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.90, 0.90, 0.90))),
            text_color: Some(Color::BLACK),
            ..Default::default()
        })
}

/// Renders an error page.
fn render_error<'window>() -> container::Container<'window, Message> {
    container(text("No body tag found").color(Color::from_rgb(1.0, 0.0, 0.0)))
        .width(Length::Fill)
        .padding(10.0)
        .style(|_| container::Style {
            background: Some(Background::Color(Color::from_rgb(0.95, 0.95, 0.95))),
            text_color: Some(Color::BLACK),
            ..Default::default()
        })
}

// TODO: Add more error/fallback pages depending on the type of issue (e.g. network errors, 404 errors)
