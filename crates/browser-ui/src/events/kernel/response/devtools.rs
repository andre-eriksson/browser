use browser_core::{DevtoolsPage, TabId};
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use iced::{Task, window::Id};
use layout::{LayoutEngine, Rect};

use crate::{
    core::{Application, Devtools, UiDevtools, WindowType},
    events::Event,
    views::devtools::window::{DevtoolsContext, DevtoolsWindow},
};

/// Handles the event when a devtools page is ready, building the style and layout trees for the page and associating it with the corresponding tab in the application.
pub(crate) fn on_devtools_page_ready(
    application: &mut Application,
    window_id: Id,
    tab_id: TabId,
    page: DevtoolsPage,
) -> Task<Event> {
    let Some(ctx) = application.browser_windows.get_mut(&window_id) else {
        tracing::warn!("Devtools page ready for unknown window id: {}", window_id);
        return Task::none();
    };

    let mut devtools_ctx = DevtoolsContext {
        viewport: DevtoolsWindow::DEFAULT_VIEWPORT_SIZE,
        page: None,
    };

    if let Some(tab) = ctx.tabs.iter_mut().find(|tab| tab.id == tab_id) {
        let abs_ctx = AbsoluteContext {
            root_font_size: 16.0,
            viewport_width: devtools_ctx.viewport.width,
            viewport_height: devtools_ctx.viewport.height,
            theme_category: application.config.preferences().theme().category,
            document_url: None,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(&abs_ctx, page.document(), page.stylesheets());
        let mut tc = ctx.text_context.lock().unwrap();
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            Rect::new(0.0, 0.0, devtools_ctx.viewport.width, devtools_ctx.viewport.height),
            &mut tc,
            None,
        );
        drop(tc);

        devtools_ctx.page = Some(UiDevtools::new(page, layout_tree));

        let (devtools_window_id, task) = application
            .window_controller
            .new_window(Some(window_id), WindowType::Devtools);

        tab.devtools = Some(Devtools {
            window_id: devtools_window_id,
            context: devtools_ctx,
        });

        task.discard()
    } else {
        tracing::warn!("Devtools page ready for unknown tab id: {}", tab_id);
        Task::none()
    }
}
