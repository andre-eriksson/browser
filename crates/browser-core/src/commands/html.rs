use std::io::Cursor;

use html_dom::DocumentRoot;
use html_parser::{HtmlStreamParser, ParserState, errors::HtmlParsingError};

use crate::Tab;

/// Parses the HTML content of the active tab for devtools inspection. This function retrieves the HTML from the active tab's document.
pub(crate) fn parse_devtools_html(active_tab: &Tab) -> Result<DocumentRoot, HtmlParsingError> {
    if active_tab.page().document().is_empty() {
        return Ok(DocumentRoot::new());
    }

    let html = active_tab.page().document().to_html();
    let mut parser = HtmlStreamParser::simple(Cursor::new(html));

    loop {
        parser.step()?;

        match parser.get_state() {
            ParserState::Completed => {
                break;
            }
            _ => continue,
        }
    }

    let result = parser.finalize();
    Ok(result.dom_tree)
}
