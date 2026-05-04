#![no_main]

use std::io::Cursor;

use html_parser::{BlockedReason, HtmlStreamParser, ParserState, errors::HtmlParsingError};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut parser = HtmlStreamParser::simple(Cursor::new(data));

    loop {
        if let Err(e) = parser.step() {
            if matches!(e, HtmlParsingError::MalformedDocument(_)) {
                // Malformed documents are expected during fuzzing, so we can ignore this error.
                break;
            }

            panic!("Error during parsing step: {}", e);
        }

        match parser.get_state() {
            ParserState::Running => continue,
            ParserState::Completed => break,
            ParserState::Blocked(reason) => match reason {
                BlockedReason::WaitingForScript(_) => {
                    if let Err(e) = parser.extract_script_content() {
                        if matches!(e, HtmlParsingError::MalformedDocument(_)) {
                            // Malformed documents are expected during fuzzing, so we can ignore this error.
                            break;
                        }

                        panic!("Error extracting script content: {}", e);
                    }
                    let _ = parser.resume();
                }
                BlockedReason::WaitingForStyle(_) => {
                    if let Err(e) = parser.extract_style_content() {
                        if matches!(e, HtmlParsingError::MalformedDocument(_)) {
                            // Malformed documents are expected during fuzzing, so we can ignore this error.
                            break;
                        }

                        panic!("Error extracting style content: {}", e);
                    }
                    let _ = parser.resume();
                }
                BlockedReason::SVGContent => {
                    if let Err(e) = parser.extract_svg_content() {
                        if matches!(e, HtmlParsingError::MalformedDocument(_)) {
                            // Malformed documents are expected during fuzzing, so we can ignore this error.
                            break;
                        }

                        panic!("Error extracting SVG content: {}", e);
                    }
                    let _ = parser.resume();
                }
                _ => {
                    panic!("Parser blocked for unhandled reason: {:?}", reason);
                }
            },
        }
    }
});
