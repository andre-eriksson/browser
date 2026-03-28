use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text::fontdb::Source;
use io::{
    Resource,
    embeded::{
        OPEN_SANS_BOLD, OPEN_SANS_EXTRA_BOLD, OPEN_SANS_LIGHT, OPEN_SANS_MEDIUM, OPEN_SANS_REGULAR,
        OPEN_SANS_SEMI_BOLD, ROBOTO_MONO_BOLD, ROBOTO_MONO_EXTRA_LIGHT, ROBOTO_MONO_LIGHT, ROBOTO_MONO_MEDIUM,
        ROBOTO_MONO_REGULAR, ROBOTO_MONO_SEMI_BOLD, ROBOTO_MONO_THIN,
    },
};

/// Load the default fonts used by the UI
pub fn load_fallback_fonts() -> [Source; 13] {
    let open_sans_light = Resource::load_embedded(OPEN_SANS_LIGHT);
    let open_sans_medium = Resource::load_embedded(OPEN_SANS_MEDIUM);
    let open_sans_regular = Resource::load_embedded(OPEN_SANS_REGULAR);
    let open_sans_semi_bold = Resource::load_embedded(OPEN_SANS_SEMI_BOLD);
    let open_sans_bold = Resource::load_embedded(OPEN_SANS_BOLD);
    let open_sans_extra_bold = Resource::load_embedded(OPEN_SANS_EXTRA_BOLD);

    let roboto_mono_thin = Resource::load_embedded(ROBOTO_MONO_THIN);
    let roboto_mono_extra_light = Resource::load_embedded(ROBOTO_MONO_EXTRA_LIGHT);
    let roboto_mono_light = Resource::load_embedded(ROBOTO_MONO_LIGHT);
    let roboto_mono_medium = Resource::load_embedded(ROBOTO_MONO_MEDIUM);
    let roboto_mono_regular = Resource::load_embedded(ROBOTO_MONO_REGULAR);
    let roboto_mono_semi_bold = Resource::load_embedded(ROBOTO_MONO_SEMI_BOLD);
    let roboto_mono_bold = Resource::load_embedded(ROBOTO_MONO_BOLD);

    [
        Source::Binary(Arc::new(open_sans_light)),
        Source::Binary(Arc::new(open_sans_medium)),
        Source::Binary(Arc::new(open_sans_regular)),
        Source::Binary(Arc::new(open_sans_semi_bold)),
        Source::Binary(Arc::new(open_sans_bold)),
        Source::Binary(Arc::new(open_sans_extra_bold)),
        Source::Binary(Arc::new(roboto_mono_thin)),
        Source::Binary(Arc::new(roboto_mono_extra_light)),
        Source::Binary(Arc::new(roboto_mono_light)),
        Source::Binary(Arc::new(roboto_mono_medium)),
        Source::Binary(Arc::new(roboto_mono_regular)),
        Source::Binary(Arc::new(roboto_mono_semi_bold)),
        Source::Binary(Arc::new(roboto_mono_bold)),
    ]
}
