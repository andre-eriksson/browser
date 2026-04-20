use std::io::Cursor;

use html_dom::DocumentRoot;
use html_parser::{HtmlStreamParser, ParserState, errors::HtmlParsingError};

/// Parses the HTML content of the active tab for devtools inspection. This function retrieves the HTML from the active tab's document.
pub fn parse_devtools_html(document: &DocumentRoot) -> Result<DocumentRoot, HtmlParsingError> {
    let html = document.to_html();
    let mut parser = HtmlStreamParser::simple(Cursor::new(html));

    loop {
        parser.step()?;

        if let ParserState::Completed = parser.get_state() {
            break;
        }
    }

    let result = parser.finalize();
    Ok(result.dom_tree)
}
