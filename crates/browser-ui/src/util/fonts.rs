use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text::fontdb::Source;
use io::embedded::{
    OPEN_SANS_BOLD, OPEN_SANS_EXTRA_BOLD, OPEN_SANS_LIGHT, OPEN_SANS_MEDIUM, OPEN_SANS_REGULAR, OPEN_SANS_SEMI_BOLD,
    ROBOTO_MONO_BOLD, ROBOTO_MONO_EXTRA_LIGHT, ROBOTO_MONO_LIGHT, ROBOTO_MONO_MEDIUM, ROBOTO_MONO_REGULAR,
    ROBOTO_MONO_SEMI_BOLD, ROBOTO_MONO_THIN, ROBOTO_SERIF_BLACK, ROBOTO_SERIF_BOLD, ROBOTO_SERIF_EXTRA_BOLD,
    ROBOTO_SERIF_EXTRA_LIGHT, ROBOTO_SERIF_LIGHT, ROBOTO_SERIF_MEDIUM, ROBOTO_SERIF_REGULAR, ROBOTO_SERIF_SEMI_BOLD,
    ROBOTO_SERIF_THIN,
};

/// Load the default fonts used by the UI
#[must_use]
pub fn load_fallback_fonts() -> [Source; 22] {
    [
        // Sans-serif
        Source::Binary(Arc::new(OPEN_SANS_BOLD.load())),
        Source::Binary(Arc::new(OPEN_SANS_EXTRA_BOLD.load())),
        Source::Binary(Arc::new(OPEN_SANS_LIGHT.load())),
        Source::Binary(Arc::new(OPEN_SANS_MEDIUM.load())),
        Source::Binary(Arc::new(OPEN_SANS_REGULAR.load())),
        Source::Binary(Arc::new(OPEN_SANS_SEMI_BOLD.load())),
        // Monospace
        Source::Binary(Arc::new(ROBOTO_MONO_BOLD.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_EXTRA_LIGHT.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_LIGHT.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_MEDIUM.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_REGULAR.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_SEMI_BOLD.load())),
        Source::Binary(Arc::new(ROBOTO_MONO_THIN.load())),
        // Serif
        Source::Binary(Arc::new(ROBOTO_SERIF_BLACK.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_BOLD.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_EXTRA_BOLD.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_EXTRA_LIGHT.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_LIGHT.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_MEDIUM.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_REGULAR.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_SEMI_BOLD.load())),
        Source::Binary(Arc::new(ROBOTO_SERIF_THIN.load())),
    ]
}
