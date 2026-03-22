use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use iced::Task;
use kernel::{DevtoolsPage, TabId};
use layout::{LayoutEngine, Rect};

use crate::{
    core::{Application, UiDevtools, WindowType},
    events::Event,
};

/// Handles the event when a devtools page is ready, building the style and layout trees for the page and associating it with the corresponding tab in the application.
pub(crate) fn on_devtools_page_ready(application: &mut Application, tab_id: TabId, page: DevtoolsPage) -> Task<Event> {
    let current_tab = application.tabs.iter_mut().find(|tab| tab.id == tab_id);

    if let Some(tab) = current_tab {
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
            document_url: None,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
        };

        let style_tree = StyleTree::build(&ctx, page.document(), page.stylesheets());
        let mut tc = application.text_context.lock().unwrap();
        let layout_tree = LayoutEngine::compute_layout(
            &style_tree,
            application
                .viewports
                .get(&application.id)
                .map(|(w, h)| Rect::new(0.0, 0.0, *w, *h))
                .unwrap_or(Rect::new(0.0, 0.0, 800.0, 600.0)),
            &mut tc,
            None,
        );
        drop(tc);

        tab.devtools_page = Some(UiDevtools::new(page, layout_tree));
    } else {
        tracing::warn!("Devtools page ready for unknown tab id: {}", tab_id);
    }

    let (_, task) = application
        .window_controller
        .new_window(WindowType::Devtools);

    task.discard()
}
