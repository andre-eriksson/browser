use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text::fontdb::Source;
use io::{
    Resource,
    embeded::{
        OPEN_SANS_BOLD, OPEN_SANS_EXTRA_BOLD, OPEN_SANS_LIGHT, OPEN_SANS_MEDIUM, OPEN_SANS_REGULAR,
        OPEN_SANS_SEMI_BOLD, ROBOTO_MONO_BOLD, ROBOTO_MONO_EXTRA_LIGHT, ROBOTO_MONO_LIGHT, ROBOTO_MONO_MEDIUM,
        ROBOTO_MONO_REGULAR, ROBOTO_MONO_SEMI_BOLD, ROBOTO_MONO_THIN, ROBOTO_SERIF_BLACK, ROBOTO_SERIF_BOLD,
        ROBOTO_SERIF_EXTRA_BOLD, ROBOTO_SERIF_EXTRA_LIGHT, ROBOTO_SERIF_LIGHT, ROBOTO_SERIF_MEDIUM,
        ROBOTO_SERIF_REGULAR, ROBOTO_SERIF_SEMI_BOLD, ROBOTO_SERIF_THIN,
    },
};

/// Load the default fonts used by the UI
pub fn load_fallback_fonts() -> [Source; 22] {
    [
        // Sans-serif
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_EXTRA_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_LIGHT))),
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_MEDIUM))),
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_REGULAR))),
        Source::Binary(Arc::new(Resource::load_embedded(OPEN_SANS_SEMI_BOLD))),
        // Monospace
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_EXTRA_LIGHT))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_LIGHT))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_MEDIUM))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_REGULAR))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_SEMI_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_MONO_THIN))),
        // Serif
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_BLACK))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_EXTRA_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_EXTRA_LIGHT))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_LIGHT))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_MEDIUM))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_REGULAR))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_SEMI_BOLD))),
        Source::Binary(Arc::new(Resource::load_embedded(ROBOTO_SERIF_THIN))),
    ]
}
