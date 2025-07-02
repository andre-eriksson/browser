use eframe::{HardwareAcceleration, NativeOptions, egui, run_simple_native};
use egui::{FontDefinitions, ThemePreference, ViewportBuilder};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    api::tabs::BrowserTab,
    content::render_content,
    html::ui::{HtmlRenderer, RendererDebugMode},
    topbar::render_top_bar,
};

/// Represents a simple web browser application using EGUI for the UI and Tokio for asynchronous networking.
///
/// # Fields
/// * `renderer` - An Arc-wrapped mutex for the HTML renderer, responsible for rendering HTML content.
/// * `tabs` - An Arc-wrapped mutex containing a vector of `BrowserTab`, representing the open tabs in the browser.
/// * `current_tab` - An Arc-wrapped mutex holding the index of the currently active tab.
/// * `next_tab_id` - An Arc-wrapped mutex holding the next unique tab ID to assign.
pub struct Browser {
    renderer: Arc<Mutex<HtmlRenderer>>,
    tabs: Arc<Mutex<Vec<BrowserTab>>>,
    current_tab: Arc<Mutex<usize>>,
    next_tab_id: Arc<Mutex<usize>>,
}

impl Browser {
    /// Creates a new instance of the `Browser` with a default starting tab.
    pub fn new() -> Self {
        let start_tab = BrowserTab::new(0, "http://localhost:8000/test.html".to_string());

        Browser {
            renderer: Arc::new(Mutex::new(HtmlRenderer::new(100, RendererDebugMode::None))),
            tabs: Arc::new(Mutex::new(vec![start_tab])),
            current_tab: Arc::new(Mutex::new(0)), // Start with the first tab
            next_tab_id: Arc::new(Mutex::new(1)), // Next tab will have ID 1
        }
    }

    /// Starts the browser application with a default viewport size and URL.
    pub fn start(&self) {
        let options = NativeOptions {
            viewport: ViewportBuilder::default().with_inner_size([1920.0, 1080.0]),
            centered: true,
            vsync: true,
            hardware_acceleration: HardwareAcceleration::Preferred,
            ..Default::default()
        };

        let cache = Arc::new(Mutex::new(HashMap::new()));
        let tabs = self.tabs.clone();
        let current_tab = self.current_tab.clone();
        let renderer = self.renderer.clone();
        let next_tab_id = self.next_tab_id.clone();

        let _ = run_simple_native("Browser", options, move |ctx, _frame| {
            initialize_fonts(ctx);
            ctx.set_theme(ThemePreference::Light);

            render_top_bar(
                ctx,
                tabs.clone(),
                current_tab.clone(),
                next_tab_id.clone(),
                &mut |tab_id| {
                    let mut all_tabs = tabs.lock().unwrap();
                    if all_tabs.len() <= 1 {
                        // Do not allow closing the last tab
                        return;
                    }

                    if let Some(pos) = all_tabs.iter().position(|t| t.id == tab_id) {
                        all_tabs[pos].close();
                        all_tabs.remove(pos);
                    }

                    // Update the current tab index if necessary
                    let mut current_tab_guard = current_tab.lock().unwrap();
                    if *current_tab_guard >= all_tabs.len() {
                        *current_tab_guard = all_tabs.len().saturating_sub(1); // Adjust current tab index
                    }
                },
            );

            let mut tabs_guard = tabs.lock().unwrap();
            let current_tab_index = *current_tab.lock().unwrap();
            if let Some(tab) = tabs_guard.get_mut(current_tab_index) {
                render_content(ctx, &cache, &renderer, tab);
            }
        });
    }
}

fn initialize_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Open Sans".to_owned(),
        egui::FontData::from_static(include_bytes!("../../resources/fonts/OpenSans-Regular.ttf"))
            .into(),
    );
    fonts.font_data.insert(
        "Roboto Mono".to_owned(),
        egui::FontData::from_static(include_bytes!(
            "../../resources/fonts/RobotoMono-Regular.ttf"
        ))
        .into(),
    );

    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "Open Sans".to_owned());

    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .insert(0, "Roboto Mono".to_owned());

    ctx.set_fonts(fonts);
}
