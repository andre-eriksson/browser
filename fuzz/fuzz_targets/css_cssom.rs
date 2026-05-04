#![no_main]

use libfuzzer_sys::fuzz_target;

use css_cssom::CSSStyleSheet;

fuzz_target!(|data: &[u8]| {
    let css_string = String::from_utf8_lossy(data);
    let _ = CSSStyleSheet::from_inline(&css_string);
});
