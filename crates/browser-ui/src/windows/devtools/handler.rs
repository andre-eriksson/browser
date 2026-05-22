use std::net::Ipv4Addr;

use browser_core::Document;
use css_display::BoxTree;
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use iced::{Size, Task, window::Id};
use layout::{ImageContext, LayoutInput, LayoutTree, Rect};
use tracing::warn;
use url::Url;

use crate::{
    core::{Application, Devtools, DevtoolsContext, DevtoolsPage, TabId, WindowType},
    events::Event,
    windows::devtools::window::DevtoolsWindow,
};

impl DevtoolsWindow {
    /// Handles the scrolling of content when a `ContentScrolled` event is received from the UI,
    /// updating the scroll offset of the active tab.
    pub fn on_scrolled(application: &mut Application, window_id: Id, x: f32, y: f32) -> Task<Event> {
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

    /// Handles the resizing of the `DevTools` window when a `Resize` event is received from the UI,
    /// updating the viewport size and recomputing the layout tree for the `DevTools` page.
    pub fn on_resized(application: &mut Application, window_id: Id, new_viewport: Size) -> Task<Event> {
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
                let localhost = Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap();

                let abs_ctx = AbsoluteContext {
                    root_font_size: 16.0,
                    viewport_width: f64::from(new_viewport.width),
                    viewport_height: f64::from(new_viewport.height),
                    theme_category: application.preferences.theme().category,
                    document_url: &localhost,
                    root_color: Color::BLACK,
                    root_line_height_multiplier: 1.2,
                };

                let style_tree =
                    StyleTree::build(Some(&application.preferences), &abs_ctx, page.dom(), page.stylesheets());
                let box_tree = BoxTree::new(page.dom(), &style_tree);
                let mut tc = ctx.text_context.lock().unwrap();
                let img_ctx = ImageContext::new();
                let layout_tree = LayoutTree::compute_layout(
                    &mut LayoutInput {
                        dom: page.dom(),
                        text: &mut tc,
                    },
                    &box_tree,
                    Rect::new(0.0, 0.0, f64::from(new_viewport.width), f64::from(new_viewport.height)),
                    &img_ctx,
                );
                drop(tc);

                page.update_layout_tree(layout_tree);
            }
        }

        Task::none()
    }

    /// Handles the event when a devtools page is ready, building the style and layout trees for the page and associating it with the corresponding tab in the application.
    pub fn on_ready(application: &mut Application, window_id: Id, tab_id: TabId, page: Document) -> Task<Event> {
        let Some(ctx) = application.browser_windows.get_mut(&window_id) else {
            warn!("Devtools page ready for unknown window id: {}", window_id);
            return Task::none();
        };

        let mut devtools_ctx = DevtoolsContext::default();

        if let Some(tab) = ctx.tab_manager.get_tab_mut(tab_id) {
            let localhost = Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap();
            let abs_ctx = AbsoluteContext {
                root_font_size: 16.0,
                viewport_width: f64::from(devtools_ctx.viewport.width),
                viewport_height: f64::from(devtools_ctx.viewport.height),
                theme_category: application.preferences.theme().category,
                document_url: &localhost,
                root_line_height_multiplier: 1.2,
                root_color: Color::BLACK,
            };

            let style_tree = StyleTree::build(Some(&application.preferences), &abs_ctx, page.dom(), page.stylesheets());
            let box_tree = BoxTree::new(page.dom(), &style_tree);
            let mut tc = ctx.text_context.lock().unwrap();
            let img_ctx = ImageContext::new();
            let layout_tree = LayoutTree::compute_layout(
                &mut LayoutInput {
                    dom: page.dom(),
                    text: &mut tc,
                },
                &box_tree,
                Rect::new(0.0, 0.0, f64::from(devtools_ctx.viewport.width), f64::from(devtools_ctx.viewport.height)),
                &img_ctx,
            );
            drop(tc);

            devtools_ctx.page = Some(DevtoolsPage::new(page, layout_tree));

            let (devtools_window_id, task) = application
                .window_controller
                .new_window(Some(window_id), WindowType::Devtools);

            tab.devtools = Some(Devtools {
                window_id: devtools_window_id,
                context: devtools_ctx,
            });

            task.discard()
        } else {
            warn!("Devtools page ready for unknown tab id: {}", tab_id);
            Task::none()
        }
    }
}
