#![no_main]

use libfuzzer_sys::fuzz_target;

use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let cursor = Cursor::new(data);
    let mut parser = HtmlStreamParser::simple(cursor);

    loop {
        let step_result = parser.step();

        match step_result {
            Ok(state) => match state {
                ParserState::Running => {}
                ParserState::Blocked(reason) => match reason {
                    BlockedReason::WaitingForScript { .. } => {}
                    BlockedReason::WaitingForStyle { .. } => {}
                    BlockedReason::SVGContent { .. } => {}
                    BlockedReason::MathML { .. } => {}
                    BlockedReason::WaitingForResource(_, _, _) => {}
                },
                ParserState::Completed(_) => {
                    break;
                }
            },
            Err(_) => break,
        }
    }
});
