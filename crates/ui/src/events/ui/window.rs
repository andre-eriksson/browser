use css_style::{AbsoluteContext, StyleTree};
use iced::{Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{
    core::{Application, WindowType},
    events::Event,
    views::devtools::window::DevtoolsWindow,
};

/// Handles the creation of a new window when a `NewWindow` event is received from the UI.
pub(crate) fn create_window(application: &mut Application, window_type: WindowType) -> Task<Event> {
    match window_type {
        WindowType::Devtools => {
            let (_, window_task) = application
                .window_controller
                .new_window(Box::new(DevtoolsWindow));
            window_task.discard()
        }
    }
}

/// Handles the closure of a window when a `CloseWindow` event is received from the UI.
pub(crate) fn close_window(application: &mut Application, window_id: Id) -> Task<Event> {
    application.window_controller.close(window_id);

    if window_id == application.id {
        if application.window_controller.open_windows.is_empty() {
            return iced::exit();
        } else {
            return application.window_controller.close_all_windows().discard();
        }
    }

    if application.window_controller.open_windows.is_empty() {
        return iced::exit();
    }

    Task::none()
}

/// Handles the resizing of a window when a `WindowResized` event is received from the UI.
pub(crate) fn on_window_resized(application: &mut Application, window_id: Id, width: f32, height: f32) -> Task<Event> {
    application.viewports.insert(window_id, (width, height));

    if window_id == application.id
        && let Some(tab) = application
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

/// Handles the change of the current URL when a `UrlChanged` event is received from the UI.
pub(crate) fn on_url_change(application: &mut Application, url: String) -> Task<Event> {
    application.current_url = url;
    Task::none()
}

/// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
/// updating the scroll offset of the active tab.
pub(crate) fn on_content_scrolled(application: &mut Application, x: f32, y: f32) -> Task<Event> {
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
