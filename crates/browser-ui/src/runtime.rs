use std::{borrow::Cow, sync::Arc};

use browser_args::BrowserArgs;
use browser_core::Browser;
use browser_preferences::BrowserPreferences;
use iced::{Font, Pixels, Settings};
use io::{
    Resource,
    embeded::{OPEN_SANS_REGULAR, ROBOTO_MONO_REGULAR},
};

use crate::{core::Application, errors::UiError};

pub struct Ui;

impl Ui {
    /// Runs the UI runtime, initializing the application and starting the event loop.
    ///
    /// # Errors
    /// If the application fails to run, a `UiError::Runtime` is returned with the underlying error.
    pub fn run(browser: Arc<Browser>, args: BrowserArgs) -> Result<(), UiError> {
        let preferences = Arc::new(BrowserPreferences::load(&args));
        let args = Arc::new(args);

        let default_font = Resource::load_embedded(OPEN_SANS_REGULAR);
        let monospace_font = Resource::load_embedded(ROBOTO_MONO_REGULAR);
        let (default_font_name, default_text_size) = {
            let theme = preferences.theme();

            (Box::leak(Box::new(theme.typography.ui.name.clone())), theme.typography.ui.size)
        };

        let result = iced::daemon(
            move || Application::new(browser.clone(), args.clone(), preferences.clone()),
            Application::update,
            Application::view,
        )
        .subscription(Application::subscriptions)
        .settings(Settings {
            fonts: vec![Cow::Owned(default_font), Cow::Owned(monospace_font)],
            default_font: Font::with_name(default_font_name),
            default_text_size: Pixels(default_text_size),
            ..Default::default()
        })
        .texture_format(Some(iced::wgpu::TextureFormat::Bgra8Unorm))
        .theme(Application::theme)
        .title(Application::title)
        .run();

        match result {
            Ok(()) => Ok(()),
            Err(error) => Err(UiError::Runtime(error)),
        }
    }
}
