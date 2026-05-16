use iced::Task;

use crate::{core::Application, events::Event, windows::browser::window::BrowserWindow};

impl BrowserWindow {
    /// Handles the change of the current URL when a `UrlChanged` event is received from the UI.
    pub fn on_url_change(application: &mut Application, window_id: iced::window::Id, url: String) -> Task<Event> {
        if let Some(ctx) = application.browser_windows.get_mut(&window_id) {
            ctx.current_url = url;
        }

        Task::none()
    }

    /// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
    /// updating the scroll offset of the active tab.
    pub fn on_scrolled(application: &mut Application, window_id: iced::window::Id, x: f32, y: f32) -> Task<Event> {
        if let Some(ctx) = application.browser_windows.get_mut(&window_id)
            && let Some(tab) = ctx.tab_manager.active_tab_mut()
        {
            tab.scroll_offset.x = x;
            tab.scroll_offset.y = y;
        }

        Task::none()
    }

    /// Handles the resizing of the browser window when a `Resize` event is received from the UI,
    /// updating the viewport size and recomputing the layout tree for the active tab's page.
    pub fn on_resized(
        application: &mut Application,
        window_id: iced::window::Id,
        new_viewport: iced::Size,
    ) -> Task<Event> {
        if let Some(ctx) = application.browser_windows.get_mut(&window_id)
            && let Some(tab) = ctx.tab_manager.active_tab_mut()
            && let Some(page_ctx) = tab.page.as_ref()
        {
            ctx.viewport = new_viewport;
            if page_ctx.document.dom().root_nodes.is_empty() {
                return Task::none();
            }

            let mut tc = ctx.text_context.lock().unwrap();

            tab.resize_current_page(new_viewport, &mut tc, &application.preferences);
        }

        Task::none()
    }
}
