use api::{dom::SharedDomNode, sender::NetworkMessage};
use eframe::{HardwareAcceleration, NativeOptions, egui, run_simple_native};
use egui::{FontDefinitions, ThemePreference, ViewportBuilder};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use crate::{
    api::tabs::BrowserTab,
    content::render_content,
    html::ui::{HtmlRenderer, RendererDebugMode},
    network::loader::NetworkLoader,
    topbar::render_top_bar,
};

/// Represents a simple web browser application using EGUI for the UI and Tokio for asynchronous networking.
///
/// # Fields
/// * `network_sender` - An unbounded sender for sending network messages to the backend.
pub struct Browser {
    network_sender: mpsc::UnboundedSender<NetworkMessage>,
    renderer: Arc<Mutex<HtmlRenderer>>,
    tabs: Arc<Mutex<Vec<BrowserTab>>>,
    current_tab: Arc<Mutex<usize>>,
}

impl Browser {
    pub fn new(network_sender: mpsc::UnboundedSender<NetworkMessage>) -> Self {
        let start_tab = BrowserTab {
            url: "http://localhost:8000/test.html".to_string(), // Default URL
            status_code: Arc::new(Mutex::new("200 OK".to_string())),
            html_content: SharedDomNode::default(),
            metadata: Default::default(),
        };

        Browser {
            network_sender,
            renderer: Arc::new(Mutex::new(HtmlRenderer::new(100, RendererDebugMode::None))),
            tabs: Arc::new(Mutex::new(vec![start_tab])),
            current_tab: Arc::new(Mutex::new(0)), // Start with the first tab
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

        let network_sender = self.network_sender.clone();
        let cache = Arc::new(Mutex::new(Default::default()));
        let tabs = self.tabs.clone();
        let current_tab = self.current_tab.clone();
        let renderer = self.renderer.clone();

        let _ = run_simple_native("Browser", options, move |ctx, _frame| {
            initialize_fonts(ctx);
            ctx.set_theme(ThemePreference::Light);

            ctx.add_image_loader(Arc::new(NetworkLoader {
                network_sender: network_sender.clone(),
                cache: cache.clone(),
            }));

            render_top_bar(
                ctx,
                &network_sender,
                &mut tabs.lock().unwrap(),
                &mut current_tab.lock().unwrap(),
            );
            render_content(
                ctx,
                &renderer,
                &mut tabs.lock().unwrap()[*current_tab.lock().unwrap()],
            );
        });

        self.network_sender
            .send(NetworkMessage::Shutdown)
            .expect("Failed to send Shutdown message");
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
