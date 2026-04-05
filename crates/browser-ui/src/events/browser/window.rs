use css_style::{AbsoluteContext, StyleTree};
use iced::{Size, Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{core::Application, events::Event};

/// Handles the change of the current URL when a `UrlChanged` event is received from the UI.
pub(crate) fn on_url_change(application: &mut Application, window_id: Id, url: String) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
        ctx.current_url = url;
    }

    Task::none()
}

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, window_id: Id, x: f32, y: f32) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id)
        && let Some(tab) = ctx.tabs.iter_mut().find(|tab| tab.id == ctx.active_tab_id)
    {
        tab.scroll_offset.x = x;
        tab.scroll_offset.y = y;
    }

    Task::none()
}

/// Handles the resizing of the browser window when a `Resize` event is received from the UI,
/// updating the viewport size and recomputing the layout tree for the active tab's page.
pub(crate) fn on_resized(application: &mut Application, window_id: Id, new_viewport: Size) -> Task<Event> {
    if let Some(ctx) = application.browser_windows.get_mut(&window_id)
        && let Some(tab) = ctx.tabs.iter_mut().find(|tab| tab.id == ctx.active_tab_id)
    {
        ctx.viewport = new_viewport;
        if tab.page.document().root_nodes.is_empty() {
            return Task::none();
        }

        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: new_viewport.width,
            viewport_height: new_viewport.height,
            theme_category: application.config.preferences().theme().category,
            ..Default::default()
        };
        let style_tree = StyleTree::build(&abs_ctx, tab.page.document(), tab.page.stylesheets());
        let image_ctx = tab.image_context();

        let mut tc = ctx.text_context.lock().unwrap();
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            Rect::new(0.0, 0.0, new_viewport.width, new_viewport.height),
            &mut tc,
            Some(&image_ctx),
        );
        drop(tc);

        tab.layout_tree = layout_tree;
    }

    Task::none()
}
