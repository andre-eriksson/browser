use std::sync::Arc;

use iced::advanced::graphics::text::cosmic_text::fontdb::Source;
use io::{
    ASSETS,
    constants::{
        OPEN_SANS_BOLD, OPEN_SANS_EXTRA_BOLD, OPEN_SANS_LIGHT, OPEN_SANS_MEDIUM, OPEN_SANS_REGULAR,
        OPEN_SANS_SEMI_BOLD, ROBOTO_MONO_BOLD, ROBOTO_MONO_EXTRA_LIGHT, ROBOTO_MONO_LIGHT,
        ROBOTO_MONO_MEDIUM, ROBOTO_MONO_REGULAR, ROBOTO_MONO_SEMI_BOLD, ROBOTO_MONO_THIN,
    },
};

/// Load the default fonts used by the UI
pub fn load_fallback_fonts() -> [Source; 13] {
    let open_sans_light = ASSETS.read().unwrap().load_embedded(OPEN_SANS_LIGHT);
    let open_sans_medium = ASSETS.read().unwrap().load_embedded(OPEN_SANS_MEDIUM);
    let open_sans_regular = ASSETS.read().unwrap().load_embedded(OPEN_SANS_REGULAR);
    let open_sans_semi_bold = ASSETS.read().unwrap().load_embedded(OPEN_SANS_SEMI_BOLD);
    let open_sans_bold = ASSETS.read().unwrap().load_embedded(OPEN_SANS_BOLD);
    let open_sans_extra_bold = ASSETS.read().unwrap().load_embedded(OPEN_SANS_EXTRA_BOLD);

    let roboto_mono_thin = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_THIN);
    let roboto_mono_extra_light = ASSETS
        .read()
        .unwrap()
        .load_embedded(ROBOTO_MONO_EXTRA_LIGHT);
    let roboto_mono_light = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_LIGHT);
    let roboto_mono_medium = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_MEDIUM);
    let roboto_mono_regular = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_REGULAR);
    let roboto_mono_semi_bold = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_SEMI_BOLD);
    let roboto_mono_bold = ASSETS.read().unwrap().load_embedded(ROBOTO_MONO_BOLD);

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
