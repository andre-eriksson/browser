#![no_main]

use html_tokenizer::HtmlTokenizer;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut tokenizer_state = html_tokenizer::TokenizerState::default();
    let mut tokens = Vec::new();

    for &byte in data {
        let ch = byte as char;
        HtmlTokenizer::process_char(&mut tokenizer_state, ch, &mut tokens);
    }
});
