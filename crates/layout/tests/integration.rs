#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;
    use css_cssom::{CSSStyleSheet, StylesheetOrigin};
    use css_style::StyleTree;
    use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
    use io::{ASSETS, constants::DEFAULT_CSS};
    use kernel::TabCollector;
    use layout::{LayoutEngine, Rect, TextContext};
    use ui::load_fallback_fonts;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    macro_rules! process_html {
        ($path:literal, $user_agent_css:expr ) => {{
            let user_agent_css = ASSETS.read().unwrap().load_embedded(DEFAULT_CSS);
            let html = include_bytes!($path);

            let mut stylesheets = if $user_agent_css {
                vec![CSSStyleSheet::from_css(
                    std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                    StylesheetOrigin::UserAgent,
                    true,
                )]
            } else {
                vec![]
            };

            let mut parser = HtmlStreamParser::<_, TabCollector>::new(html.as_slice(), None, None);

            loop {
                match parser.step() {
                    Ok(_) => match parser.get_state() {
                        ParserState::Running => continue,
                        ParserState::Blocked(reason) => match reason {
                            BlockedReason::WaitingForStyle(_attributes) => {
                                if let Ok(css) = parser.extract_style_content() {
                                    let stylesheet = CSSStyleSheet::from_css(
                                        &css,
                                        StylesheetOrigin::Author,
                                        true,
                                    );
                                    stylesheets.push(stylesheet);
                                }

                                parser.resume().unwrap();
                            }
                            _ => {
                                panic!("Test files will only block on styles.");
                            }
                        },
                        ParserState::Completed => {
                            break;
                        }
                    },
                    Err(e) => panic!("Parser error: {:?}", e),
                }
            }

            let result = parser.finalize();
            let document = result.dom_tree;
            let style_tree = StyleTree::build(&document, &stylesheets);
            let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
            let mut text_context = TextContext::new(font_system);
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut text_context)
        }};
    }

    #[test]
    fn test_collapsing() {
        let layout = process_html!("fixtures/collapsing.html", false);

        let root = &layout.root_nodes[0];
        assert_eq!(root.dimensions.x, 0.0);
        assert_eq!(root.dimensions.y, 0.0);
        assert_eq!(root.dimensions.height, 400.0);
        assert_eq!(root.dimensions.height, layout.content_height);

        let body = &root.children[0];
        assert_eq!(body.dimensions.x, 8.0);
        assert_eq!(body.dimensions.y, 20.0);
        assert_eq!(body.dimensions.height, 280.0);
        assert_eq!(body.dimensions.width, 784.0);
        assert_eq!(body.children.len(), 4);

        let first_div = &body.children[0];
        assert_eq!(first_div.dimensions.x, 28.0);
        assert_eq!(first_div.dimensions.y, 20.0);
        assert_eq!(first_div.dimensions.height, 30.0);
        assert_eq!(first_div.dimensions.width, 744.0);
        assert_eq!(first_div.resolved_margin.top, 20.0);
        assert_eq!(
            first_div.resolved_margin.top,
            first_div.resolved_margin.bottom
        );

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 28.0);
        assert_eq!(second_div.dimensions.y, 70.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 744.0);
        assert_eq!(second_div.resolved_padding.top, 10.0);
        assert_eq!(second_div.resolved_padding.bottom, 10.0);
        assert_eq!(second_div.resolved_margin.top, 20.0);
        assert_eq!(
            second_div.resolved_margin.top,
            first_div.resolved_margin.bottom
        );

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 28.0);
        assert_eq!(third_div.dimensions.y, 140.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 744.0);
        assert_eq!(third_div.resolved_margin.top, 20.0);
        assert_eq!(
            third_div.resolved_margin.top,
            second_div.resolved_margin.bottom
        );

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 108.0);
        assert_eq!(fourth_div.dimensions.y, 270.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 584.0);
        assert_eq!(fourth_div.resolved_margin.top, 100.0);
        assert_eq!(
            fourth_div.resolved_margin.top,
            fourth_div.resolved_margin.bottom
        );
    }

    #[test]
    fn test_collapsing_padding() {
        let layout = process_html!("fixtures/collapsing_padding.html", false);

        let root = &layout.root_nodes[0];
        assert_eq!(root.dimensions.x, 0.0);
        assert_eq!(root.dimensions.y, 0.0);
        assert_eq!(root.dimensions.height, 436.0);
        assert_eq!(root.dimensions.height, layout.content_height);

        let body = &root.children[0];
        assert_eq!(body.dimensions.x, 8.0);
        assert_eq!(body.dimensions.y, 8.0);
        assert_eq!(body.dimensions.height, 420.0);
        assert_eq!(body.dimensions.width, 784.0);
        assert_eq!(body.children.len(), 4);

        let first_div = &body.children[0];
        assert_eq!(first_div.dimensions.x, 38.0);
        assert_eq!(first_div.dimensions.y, 38.0);
        assert_eq!(first_div.dimensions.height, 30.0);
        assert_eq!(first_div.dimensions.width, 704.0);
        assert_eq!(first_div.resolved_margin.top, 20.0);
        assert_eq!(
            first_div.resolved_margin.top,
            first_div.resolved_margin.bottom
        );

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 38.0);
        assert_eq!(second_div.dimensions.y, 88.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 704.0);
        assert_eq!(second_div.resolved_padding.top, 10.0);
        assert_eq!(second_div.resolved_padding.bottom, 10.0);
        assert_eq!(second_div.resolved_margin.top, 20.0);
        assert_eq!(
            second_div.resolved_margin.top,
            first_div.resolved_margin.bottom
        );

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 38.0);
        assert_eq!(third_div.dimensions.y, 158.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 704.0);
        assert_eq!(third_div.resolved_margin.top, 20.0);
        assert_eq!(
            third_div.resolved_margin.top,
            second_div.resolved_margin.bottom
        );

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 118.0);
        assert_eq!(fourth_div.dimensions.y, 288.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 544.0);
        assert_eq!(fourth_div.resolved_margin.top, 100.0);
        assert_eq!(
            fourth_div.resolved_margin.top,
            fourth_div.resolved_margin.bottom
        );
    }

    #[test]
    fn test_mixed_content() {
        let layout = process_html!("fixtures/mixed.html", false);

        let root = &layout.root_nodes[0];
        assert_eq!(root.dimensions.x, 0.0);
        assert_eq!(root.dimensions.y, 0.0);
        assert_eq!(root.dimensions.height, 176.0);
        assert_eq!(root.dimensions.height, layout.content_height);

        let body = &root.children[0];
        assert_eq!(body.dimensions.x, 8.0);
        assert_eq!(body.dimensions.y, 8.0);
        assert_eq!(body.dimensions.height, 148.0);
        assert_eq!(body.children.len(), 3);

        let first_span = &body.children[0];
        assert_eq!(first_span.dimensions.x, 8.0);
        assert_eq!(first_span.dimensions.y, 8.0);
        assert_eq!(first_span.dimensions.height, 24.0);
        assert!(first_span.dimensions.width > 90.0 && first_span.dimensions.width < 100.0);

        let first_div = &body.children[1];
        assert_eq!(first_div.dimensions.x, 28.0);
        assert_eq!(first_div.dimensions.y, 52.0);
        assert_eq!(first_div.dimensions.height, 30.0);
        assert_eq!(first_div.dimensions.width, 744.0);

        // <br> is just adjusting the y position of the next element

        let second_div = &body.children[2];
        assert_eq!(second_div.dimensions.x, 28.0);
        assert_eq!(second_div.dimensions.y, 126.0);
        assert_eq!(second_div.dimensions.height, 30.0);
        assert_eq!(second_div.dimensions.width, 744.0);
    }
}
