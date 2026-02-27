#[cfg(test)]
mod tests {
    use cosmic_text::FontSystem;
    use css_cssom::{CSSStyleSheet, StylesheetOrigin};
    use css_style::{AbsoluteContext, StyleTree};
    use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
    use io::{Resource, embeded::DEFAULT_CSS};
    use kernel::TabCollector;
    use layout::{ImageContext, LayoutEngine, Rect, TextContext};
    use ui::load_fallback_fonts;

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    /// Parses an HTML fixture and builds a `StyleTree` + `TextContext` without
    /// running layout.  This is the building block for the other macros.
    macro_rules! process_html_raw {
        ($path:literal, $user_agent_css:expr) => {{
            let user_agent_css = Resource::load_embedded(DEFAULT_CSS);
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
                                    let stylesheet = CSSStyleSheet::from_css(&css, StylesheetOrigin::Author, true);
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

            let absolute_ctx = AbsoluteContext {
                viewport_width: viewport().width,
                viewport_height: viewport().height,
                ..Default::default()
            };

            let result = parser.finalize();
            let document = result.dom_tree;
            let style_tree = StyleTree::build(&absolute_ctx, &document, &stylesheets);
            let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
            let text_context = TextContext::new(font_system);
            (style_tree, text_context)
        }};
    }

    /// Runs layout from a pre-built `StyleTree`, optionally with an
    /// `ImageContext` for known image dimensions.
    macro_rules! layout_from {
        ($style_tree:expr, $text_context:expr) => {{ LayoutEngine::compute_layout(&$style_tree, viewport(), $text_context, None) }};
        ($style_tree:expr, $text_context:expr, $image_ctx:expr) => {{ LayoutEngine::compute_layout(&$style_tree, viewport(), $text_context, Some($image_ctx)) }};
    }

    /// Convenience: parse HTML and immediately compute layout (no image
    /// context).
    macro_rules! process_html {
        ($path:literal, $user_agent_css:expr) => {{
            let (style_tree, mut text_context) = process_html_raw!($path, $user_agent_css);
            LayoutEngine::compute_layout(&style_tree, viewport(), &mut text_context, None)
        }};
    }

    #[test]
    fn test_collapsing() {
        let layout = process_html!("fixtures/collapsing.html", true);

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
        assert_eq!(first_div.resolved_margin.top, first_div.resolved_margin.bottom);

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 28.0);
        assert_eq!(second_div.dimensions.y, 70.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 744.0);
        assert_eq!(second_div.resolved_padding.top, 10.0);
        assert_eq!(second_div.resolved_padding.bottom, 10.0);
        assert_eq!(second_div.resolved_margin.top, 20.0);
        assert_eq!(second_div.resolved_margin.top, first_div.resolved_margin.bottom);

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 28.0);
        assert_eq!(third_div.dimensions.y, 140.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 744.0);
        assert_eq!(third_div.resolved_margin.top, 20.0);
        assert_eq!(third_div.resolved_margin.top, second_div.resolved_margin.bottom);

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 108.0);
        assert_eq!(fourth_div.dimensions.y, 270.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 584.0);
        assert_eq!(fourth_div.resolved_margin.top, 100.0);
        assert_eq!(fourth_div.resolved_margin.top, fourth_div.resolved_margin.bottom);
    }

    #[test]
    fn test_collapsing_padding() {
        let layout = process_html!("fixtures/collapsing_padding.html", true);

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
        assert_eq!(first_div.dimensions.width, 724.0);
        assert_eq!(first_div.resolved_margin.top, 20.0);
        assert_eq!(first_div.resolved_margin.top, first_div.resolved_margin.bottom);

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 38.0);
        assert_eq!(second_div.dimensions.y, 88.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 724.0);
        assert_eq!(second_div.resolved_padding.top, 10.0);
        assert_eq!(second_div.resolved_padding.bottom, 10.0);
        assert_eq!(second_div.resolved_margin.top, 20.0);
        assert_eq!(second_div.resolved_margin.top, first_div.resolved_margin.bottom);

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 38.0);
        assert_eq!(third_div.dimensions.y, 158.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 724.0);
        assert_eq!(third_div.resolved_margin.top, 20.0);
        assert_eq!(third_div.resolved_margin.top, second_div.resolved_margin.bottom);

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 118.0);
        assert_eq!(fourth_div.dimensions.y, 288.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 564.0);
        assert_eq!(fourth_div.resolved_margin.top, 100.0);
        assert_eq!(fourth_div.resolved_margin.top, fourth_div.resolved_margin.bottom);
    }

    #[test]
    fn test_mixed_content() {
        let layout = process_html!("fixtures/mixed.html", true);

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

    #[test]
    fn test_child_calc_percentage_resolves_against_parent_size() {
        let layout = process_html!("fixtures/calc_percent_child.html", true);

        let root = &layout.root_nodes[0];
        let body = &root.children[0];
        let first_div = &body.children[0];
        let child = &first_div.children[0];

        assert_eq!(child.dimensions.width, 260.0);
        assert_eq!(child.dimensions.height, 260.0);
    }

    /// Verifies that the relayout system correctly repositions siblings and
    /// updates parent heights when an image's intrinsic dimensions become
    /// known after the initial layout.
    ///
    /// Flow:
    ///   1. Initial layout with placeholder image dimensions (300×150).
    ///   2. Relayout with known intrinsic dimensions (640×480) via
    ///      `ImageContext`.
    ///   3. Assert that the "After image" paragraph moved down and the
    ///      container grew taller.
    #[test]
    fn test_image_relayout_repositions_siblings() {
        let (style_tree, mut text_context) = process_html_raw!("fixtures/image_relayout.html", true);

        let layout_before = layout_from!(style_tree, &mut text_context);

        let root = &layout_before.root_nodes[0];
        let body = &root.children[0];

        let container = &body.children[0];
        let img_node = container
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .expect("should have an image node");

        assert_eq!(img_node.dimensions.width, 300.0);
        assert_eq!(img_node.dimensions.height, 150.0);
        assert!(
            img_node
                .image_data
                .as_ref()
                .unwrap()
                .image_needs_intrinsic_size
        );

        let after_p_y_before = container
            .children
            .last()
            .expect("container should have children")
            .dimensions
            .y;
        let content_height_before = layout_before.content_height;

        let mut image_ctx = ImageContext::new();
        image_ctx.insert("https://example.com/test.png", 640.0, 480.0);

        let layout_after = layout_from!(style_tree, &mut text_context, &image_ctx);

        let root2 = &layout_after.root_nodes[0];
        let body2 = &root2.children[0];
        let container2 = &body2.children[0];

        let img_node2 = container2
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .expect("should still have an image node");

        assert_eq!(img_node2.dimensions.width, 640.0);
        assert_eq!(img_node2.dimensions.height, 480.0);
        assert!(
            !img_node2
                .image_data
                .as_ref()
                .unwrap()
                .image_needs_intrinsic_size
        );

        let after_p_y_after = container2
            .children
            .last()
            .expect("container should have children")
            .dimensions
            .y;

        assert!(
            after_p_y_after > after_p_y_before,
            "After-image paragraph should move down after relayout (before: {}, after: {})",
            after_p_y_before,
            after_p_y_after,
        );

        assert!(
            layout_after.content_height > content_height_before,
            "Total content height should increase after relayout (before: {}, after: {})",
            content_height_before,
            layout_after.content_height,
        );
    }

    /// Verifies that relayout with the same `ImageContext` is idempotent –
    /// running it twice produces identical results.
    #[test]
    fn test_image_relayout_is_idempotent() {
        let (style_tree, mut text_context) = process_html_raw!("fixtures/image_relayout.html", true);

        let mut image_ctx = ImageContext::new();
        image_ctx.insert("https://example.com/test.png", 640.0, 480.0);

        let layout_a = layout_from!(style_tree, &mut text_context, &image_ctx);
        let layout_b = layout_from!(style_tree, &mut text_context, &image_ctx);

        assert_eq!(layout_a.content_height, layout_b.content_height);

        let img_a = layout_a.root_nodes[0].children[0].children[0]
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .unwrap();
        let img_b = layout_b.root_nodes[0].children[0].children[0]
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .unwrap();

        assert_eq!(img_a.dimensions.width, img_b.dimensions.width);
        assert_eq!(img_a.dimensions.height, img_b.dimensions.height);
        assert_eq!(img_a.dimensions.x, img_b.dimensions.x);
        assert_eq!(img_a.dimensions.y, img_b.dimensions.y);
    }
}
