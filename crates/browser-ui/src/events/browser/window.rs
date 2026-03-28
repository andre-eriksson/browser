use css_style::{AbsoluteContext, StyleTree};
use iced::{Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{core::Application, events::Event};

/// Handles the change of the current URL when a `UrlChanged` event is received from the UI.
pub(crate) fn on_url_change(application: &mut Application, url: String) -> Task<Event> {
    application.current_url = url;
    Task::none()
}

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, x: f32, y: f32) -> Task<Event> {
    if let Some(tab) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
    {
        tab.scroll_offset.x = x;
        tab.scroll_offset.y = y;
    }

    Task::none()
}

/// Handles the resizing of the browser window when a `Resize` event is received from the UI,
/// updating the viewport size and recomputing the layout tree for the active tab's page.
pub(crate) fn on_resized(application: &mut Application, window_id: Id, width: f32, height: f32) -> Task<Event> {
    application.viewports.insert(window_id, (width, height));

    if let Some(tab) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
    {
        let ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: application
                .viewports
                .get(&application.id)
                .map(|(w, _)| *w)
                .unwrap_or(800.0),
            viewport_height: application
                .viewports
                .get(&application.id)
                .map(|(_, h)| *h)
                .unwrap_or(600.0),
            theme_category: application.config.preferences().active_theme().category,
            ..Default::default()
        };
        let style_tree = StyleTree::build(&ctx, tab.page.document(), tab.page.stylesheets());
        let image_ctx = tab.image_context();

        let mut tc = application.text_context.lock().unwrap();
        let layout_tree =
            LayoutEngine::compute_layout(&style_tree, Rect::new(0.0, 0.0, width, height), &mut tc, Some(&image_ctx));
        drop(tc);

        tab.layout_tree = layout_tree;
    }

    Task::none()
}
