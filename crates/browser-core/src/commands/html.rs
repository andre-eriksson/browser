use std::io::{Cursor, Write};

use html_dom::{DocumentRoot, DomNode, NodeData};
use html_parser::{HtmlStreamParser, ParserState, errors::HtmlParsingError};

/// Parses the HTML content of the active tab for devtools inspection. This function retrieves the HTML from the active tab's document.
pub fn parse_devtools_html(title: &str, document: &DocumentRoot) -> Result<DocumentRoot, HtmlParsingError> {
    fn node_to_html(mut html: &mut Vec<u8>, node: &DomNode, dom_tree: &DocumentRoot, depth: usize) {
        if node.data.as_text().is_some_and(|t| t.trim().is_empty()) {
            return;
        }

        write!(&mut html, "<div class='line'>").unwrap();
        write!(&mut html, "<span style='margin-left: calc({depth} * 2rem)'></span>").unwrap();

        match &node.data {
            NodeData::Element(elem) => {
                write!(&mut html, "<span class='tag'>&lt;</span><span class='tag-name'>{}</span>", elem.tag).unwrap();

                write!(
                    &mut html,
                    " <span class='attr-name'>data-node-id</span><span class='attr-equals'>=</span><span class='attr-value'>\"{}\"</span>",
                    node.id,
                ).unwrap();

                if let Some(attrs) = &elem.attributes {
                    for (attr_name, attr_value) in attrs {
                        if attr_name.trim().is_empty() {
                            continue;
                        }

                        write!(
                            &mut html,
                            " <span class='attr-name'>{attr_name}</span><span class='attr-equals'>=</span><span class='attr-value'>\"{attr_value}\"</span>",
                        ).unwrap();
                    }
                }

                write!(&mut html, "<span class='tag'>&gt;</span>").unwrap();

                let has_child = !node.children.is_empty();

                for child_id in &node.children {
                    node_to_html(html, &dom_tree[child_id], dom_tree, depth + 1);
                }

                if has_child {
                    write!(&mut html, "<span style='margin-left: calc({depth} * 2rem)'></span>").unwrap();
                }

                if !elem.tag.is_void_element() {
                    write!(
                        &mut html,
                        "<span class='tag'>&lt;/</span><span class='tag-name'>{}</span><span class='tag'>&gt;</span>",
                        elem.tag
                    )
                    .unwrap();
                }
            }
            NodeData::Text(text) => {
                write!(&mut html, "<span class='text'>{text}</span>").unwrap();
            }
        }

        write!(&mut html, "</div>").unwrap();
    }

    let mut html = Vec::new();
    write!(&mut html, "<html><head></head><body>").unwrap();

    write!(&mut html, "<header>").unwrap();
    write!(&mut html, "<h3>Devtools - {}</h3>", title).unwrap();
    write!(&mut html, "<nav>").unwrap();

    write!(&mut html, "<div data-active>").unwrap();
    write!(&mut html, "<p>DOM</p>").unwrap();
    write!(&mut html, "</div>").unwrap();

    write!(&mut html, "</nav>").unwrap();
    write!(&mut html, "</header>").unwrap();

    write!(&mut html, "<main id=\"document_html\">").unwrap();
    for root_id in &document.root_nodes {
        node_to_html(&mut html, &document[root_id], document, 0);
    }
    write!(&mut html, "</main>").unwrap();
    write!(&mut html, "</body></html>").unwrap();

    let mut parser = HtmlStreamParser::simple(Cursor::new(html));

    let result = loop {
        if let ParserState::Completed(result) = parser.step()? {
            break result;
        }
    };

    Ok(result.dom_tree)
}
