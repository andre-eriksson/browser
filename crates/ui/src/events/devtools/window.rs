use css_style::{AbsoluteContext, StyleTree};
use iced::{Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{core::Application, events::Event};

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, x: f32, y: f32) -> Task<Event> {
    if let Some(devtools) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
        .and_then(|t| t.devtools_page.as_mut())
    {
        devtools.scroll_offset.x = x;
        devtools.scroll_offset.y = y;
    }

    Task::none()
}

/// Handles the resizing of the DevTools window when a `Resize` event is received from the UI,
/// updating the viewport size and recomputing the layout tree for the DevTools page.
pub(crate) fn on_resized(application: &mut Application, window_id: Id, width: f32, height: f32) -> Task<Event> {
    application.viewports.insert(window_id, (width, height));

    if let Some(devtools) = application
        .tabs
        .iter_mut()
        .find(|tab| tab.id == application.active_tab)
        .and_then(|t| t.devtools_page.as_mut())
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
            theme_category: application.config.active_theme().category,
            ..Default::default()
        };
        let style_tree = StyleTree::build(&ctx, devtools.document(), devtools.stylesheets());

        let mut tc = application.text_context.lock().unwrap();
        let layout_tree = LayoutEngine::compute_layout(&style_tree, Rect::new(0.0, 0.0, width, height), &mut tc, None);
        drop(tc);

        devtools.layout_tree = layout_tree;
    }

    Task::none()
}
