use css_style::{AbsoluteContext, StyleTree};
use iced::{Size, Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{core::Application, events::Event};

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_scrolled(application: &mut Application, window_id: Id, x: f32, y: f32) -> Task<Event> {
    let devtools_window = application
        .window_controller
        .get_window(window_id)
        .expect("Devtools window should exist for scroll events");

    if let Some(ctx) = application.browser_windows.get_mut(
        &devtools_window
            .parent_id()
            .expect("The Devtools should have a parent browser window"),
    ) && let Some(devtools) = ctx
        .tab_manager
        .active_tab_mut()
        .and_then(|t| t.devtools.as_mut())
        .and_then(|d| d.context.page.as_mut())
    {
        devtools.scroll_offset.x = x;
        devtools.scroll_offset.y = y;
    }

    Task::none()
}

/// Handles the resizing of the DevTools window when a `Resize` event is received from the UI,
/// updating the viewport size and recomputing the layout tree for the DevTools page.
pub(crate) fn on_resized(application: &mut Application, window_id: Id, new_viewport: Size) -> Task<Event> {
    let devtools_window = application
        .window_controller
        .get_window(window_id)
        .expect("Devtools window should exist for resize events");

    if let Some(ctx) = application.browser_windows.get_mut(
        &devtools_window
            .parent_id()
            .expect("The Devtools should have a parent browser window"),
    ) && let Some(devtools) = ctx
        .tab_manager
        .active_tab_mut()
        .and_then(|t| t.devtools.as_mut())
    {
        devtools.context.viewport = new_viewport;

        if let Some(page) = devtools.context.page.as_mut() {
            let abs_ctx = AbsoluteContext {
                root_font_size: 16.0,
                viewport_width: new_viewport.width,
                viewport_height: new_viewport.height,
                theme_category: application.config.preferences().theme().category,
                ..Default::default()
            };
            let style_tree = StyleTree::build(&abs_ctx, page.document(), page.stylesheets());

            let mut tc = ctx.text_context.lock().unwrap();
            let layout_tree = LayoutEngine::compute_layout(
                &style_tree,
                Rect::new(0.0, 0.0, new_viewport.width, new_viewport.height),
                &mut tc,
                None,
            );
            drop(tc);

            page.layout_tree = layout_tree;
        }
    }

    Task::none()
}
