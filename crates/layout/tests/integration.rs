#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Cursor, Read},
    };

    use std::net::Ipv4Addr;
    use url::Url;

    use browser_config::BrowserConfig;
    use browser_preferences::theme::ThemeCategory;
    use browser_ui::load_fallback_fonts;
    use cosmic_text::FontSystem;
    use css_cssom::{CSSStyleSheet, StylesheetOrigin};
    use css_style::{AbsoluteContext, StyleTree};
    use css_values::color::Color;
    use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
    use io::{Resource, embeded::DEFAULT_CSS};
    use layout::{ImageContext, LayoutEngine, Rect, TextContext};

    fn load_fixture(html: &str) -> String {
        let file = File::open(format!("tests/fixtures/{}", html)).expect("failed to open fixture");
        let mut decoder = zstd::Decoder::new(file).expect("failed to create decoder");

        let mut s = String::new();
        decoder
            .read_to_string(&mut s)
            .expect("failed to decompress");
        s
    }

    #[allow(
        dead_code,
        reason = "Some tests may want to load uncompressed HTML for easier debugging."
    )]
    fn load_fixture_raw(html: &str) -> String {
        std::fs::read_to_string(format!("tests/fixtures/{}", html)).expect("failed to read fixture")
    }

    fn viewport() -> Rect {
        Rect::new(0.0, 0.0, 800.0, 600.0)
    }

    /// Parses an HTML fixture and builds a `StyleTree` + `TextContext` without
    /// running layout.  This is the building block for the other macros.
    macro_rules! process_html_raw {
        ($path:literal, $user_agent_css:expr) => {{
            let config = BrowserConfig::default();
            let user_agent_css = Resource::load_embedded(DEFAULT_CSS);
            let html = load_fixture($path);

            let mut stylesheets = if $user_agent_css {
                vec![CSSStyleSheet::from_css(
                    std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                    StylesheetOrigin::UserAgent,
                    true,
                )]
            } else {
                vec![]
            };

            let cursor = Cursor::new(html);
            let reader = BufReader::new(cursor);

            let mut parser = HtmlStreamParser::simple(reader);

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

            let url = Box::leak(Box::new(Url::parse(&format!("http://{}", Ipv4Addr::LOCALHOST)).unwrap()));

            let absolute_ctx = AbsoluteContext {
                viewport_width: viewport().width,
                viewport_height: viewport().height,
                root_font_size: 16.0,
                root_line_height_multiplier: 1.2,
                document_url: &url,
                theme_category: ThemeCategory::Light,
                root_color: Color::BLACK,
            };

            let result = parser.finalize();
            let document = result.dom_tree;
            let style_tree = StyleTree::build(&config, &absolute_ctx, &document, &stylesheets);
            let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
            let text_context = TextContext::new(font_system);
            (document, style_tree, text_context)
        }};
    }

    /// Runs layout from a pre-built `StyleTree`, optionally with an
    /// `ImageContext` for known image dimensions.
    macro_rules! layout_from {
        ($dom_tree:expr, $style_tree:expr, $text_context:expr) => {{
            let img_ctx = ImageContext::new();
            LayoutEngine::compute_layout(&$dom_tree, &$style_tree, viewport(), $text_context, &img_ctx)
        }};
        ($dom_tree:expr, $style_tree:expr, $text_context:expr, $image_ctx:expr) => {{ LayoutEngine::compute_layout(&$dom_tree, &$style_tree, viewport(), $text_context, $image_ctx) }};
    }

    /// Convenience: parse HTML and immediately compute layout (no image
    /// context).
    macro_rules! process_html {
        ($path:literal, $user_agent_css:expr) => {{
            let (dom_tree, style_tree, mut text_context) = process_html_raw!($path, $user_agent_css);
            let img_ctx = ImageContext::new();
            LayoutEngine::compute_layout(&dom_tree, &style_tree, viewport(), &mut text_context, &img_ctx)
        }};
    }

    /// Same as process_html_raw but for uncompressed HTML files.
    #[allow(
        unused_macros,
        reason = "Some tests may want to load uncompressed HTML for easier debugging."
    )]
    macro_rules! process_html_uncompressed {
        ($path:literal, $user_agent_css:expr) => {{
            let user_agent_css = Resource::load_embedded(DEFAULT_CSS);
            let html = load_fixture_raw($path);

            let mut stylesheets = if $user_agent_css {
                vec![CSSStyleSheet::from_css(
                    std::str::from_utf8(&user_agent_css).unwrap_or_default(),
                    StylesheetOrigin::UserAgent,
                    true,
                )]
            } else {
                vec![]
            };

            let cursor = Cursor::new(html);
            let reader = BufReader::new(cursor);

            let mut parser = HtmlStreamParser::simple(reader);

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
                root_font_size: 16.0,
                root_line_height_multiplier: 1.2,
                ..Default::default()
            };

            let result = parser.finalize();
            let document = result.dom_tree;
            let style_tree = StyleTree::build(&absolute_ctx, &document, &stylesheets);
            let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
            let mut text_context = TextContext::new(font_system);
            let img_ctx = ImageContext::new();
            LayoutEngine::compute_layout(&document, &style_tree, viewport(), &mut text_context, &img_ctx)
        }};
    }

    #[test]
    fn test_collapsing() {
        let layout = process_html!("collapsing.html.zst", true);

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
        assert_eq!(first_div.margin.top.to_px(), 20.0);
        assert_eq!(first_div.margin.top, first_div.margin.bottom);

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 28.0);
        assert_eq!(second_div.dimensions.y, 70.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 744.0);
        assert_eq!(second_div.padding.top, 10.0);
        assert_eq!(second_div.padding.bottom, 10.0);
        assert_eq!(second_div.margin.top.to_px(), 20.0);
        assert_eq!(second_div.margin.top, first_div.margin.bottom);

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 28.0);
        assert_eq!(third_div.dimensions.y, 140.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 744.0);
        assert_eq!(third_div.margin.top.to_px(), 20.0);
        assert_eq!(third_div.margin.top, second_div.margin.bottom);

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 108.0);
        assert_eq!(fourth_div.dimensions.y, 270.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 584.0);
        assert_eq!(fourth_div.margin.top.to_px(), 100.0);
        assert_eq!(fourth_div.margin.top, fourth_div.margin.bottom);
    }

    #[test]
    fn test_collapsing_padding() {
        let layout = process_html!("collapsing_padding.html.zst", true);

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
        assert_eq!(first_div.margin.top.to_px(), 20.0);
        assert_eq!(first_div.margin.top, first_div.margin.bottom);

        let second_div = &body.children[1];
        assert_eq!(second_div.dimensions.x, 38.0);
        assert_eq!(second_div.dimensions.y, 88.0);
        assert_eq!(second_div.dimensions.height, 50.0);
        assert_eq!(second_div.dimensions.width, 724.0);
        assert_eq!(second_div.padding.top, 10.0);
        assert_eq!(second_div.padding.bottom, 10.0);
        assert_eq!(second_div.margin.top.to_px(), 20.0);
        assert_eq!(second_div.margin.top, first_div.margin.bottom);

        let third_div = &body.children[2];
        assert_eq!(third_div.dimensions.x, 38.0);
        assert_eq!(third_div.dimensions.y, 158.0);
        assert_eq!(third_div.dimensions.height, 30.0);
        assert_eq!(third_div.dimensions.width, 724.0);
        assert_eq!(third_div.margin.top.to_px(), 20.0);
        assert_eq!(third_div.margin.top, second_div.margin.bottom);

        let fourth_div = &body.children[3];
        assert_eq!(fourth_div.dimensions.x, 118.0);
        assert_eq!(fourth_div.dimensions.y, 288.0);
        assert_eq!(fourth_div.dimensions.height, 30.0);
        assert_eq!(fourth_div.dimensions.width, 564.0);
        assert_eq!(fourth_div.margin.top.to_px(), 100.0);
        assert_eq!(fourth_div.margin.top, fourth_div.margin.bottom);
    }

    #[test]
    fn test_mixed_content() {
        let layout = process_html!("mixed.html.zst", true);

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
        let layout = process_html!("calc_percent_child.html.zst", true);

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
        let (dom, style_tree, mut text_context) = process_html_raw!("image_relayout.html.zst", true);

        let mut layout = layout_from!(dom, style_tree, &mut text_context);

        let root = &layout.root_nodes[0];
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
        let content_height_before = layout.content_height;

        let mut image_ctx = ImageContext::new();
        image_ctx.insert("https://example.com/test.png", 640.0, 480.0);

        LayoutEngine::relayout_node(
            img_node.node_id,
            Rect::default(),
            &mut layout,
            &style_tree,
            &dom,
            &mut text_context,
            &image_ctx,
        );

        let root2 = &layout.root_nodes[0];
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
            layout.content_height > content_height_before,
            "Total content height should increase after relayout (before: {}, after: {})",
            content_height_before,
            layout.content_height,
        );
    }

    #[test]
    fn test_image_relayout_respects_css_width_percentage() {
        let (dom, style_tree, mut text_context) = process_html_raw!("image_relayout_css_width.html.zst", true);

        let mut layout = layout_from!(dom, style_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let body = &root.children[0];
        let container = &body.children[0];
        let img_node = container
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .expect("should have an image node");

        let mut image_ctx = ImageContext::new();
        image_ctx.insert("https://example.com/test.png", 640.0, 480.0);

        LayoutEngine::relayout_node(
            img_node.node_id,
            Rect::default(),
            &mut layout,
            &style_tree,
            &dom,
            &mut text_context,
            &image_ctx,
        );

        let root2 = &layout.root_nodes[0];
        let body2 = &root2.children[0];
        let container2 = &body2.children[0];
        let img_node2 = container2
            .children
            .iter()
            .find(|n| n.image_data.is_some())
            .expect("should still have an image node");

        assert_eq!(img_node2.dimensions.width, container2.dimensions.width);
        assert_eq!(img_node2.dimensions.width, 784.0);
        assert_eq!(img_node2.dimensions.height, 588.0);
    }

    /// Verifies that relayout with the same `ImageContext` is idempotent –
    /// running it twice produces identical results.
    #[test]
    fn test_image_relayout_is_idempotent() {
        let (dom_tree, style_tree, mut text_context) = process_html_raw!("image_relayout.html.zst", true);

        let mut image_ctx = ImageContext::new();
        image_ctx.insert("https://example.com/test.png", 640.0, 480.0);

        let layout_a = layout_from!(dom_tree, style_tree, &mut text_context, &image_ctx);
        let layout_b = layout_from!(dom_tree, style_tree, &mut text_context, &image_ctx);

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

    #[test]
    fn test_float_basic() {
        let (dom_tree, style_tree, mut text_context) = process_html_raw!("float_basic.html.zst", true);
        let layout = layout_from!(dom_tree, style_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let body = &root.children[0];
        assert_eq!(body.children.len(), 3);

        let float_left = &body.children[0];
        assert_eq!(float_left.dimensions.x, 0.0);
        assert_eq!(float_left.dimensions.y, 0.0);
        assert_eq!(float_left.dimensions.width, 100.0);
        assert_eq!(float_left.dimensions.height, 100.0);

        let float_right = &body.children[1];
        assert_eq!(float_right.dimensions.x, 700.0);
        assert_eq!(float_right.dimensions.y, 0.0);
        assert_eq!(float_right.dimensions.width, 100.0);
        assert_eq!(float_right.dimensions.height, 100.0);

        let block = &body.children[2];
        assert_eq!(block.dimensions.x, 0.0);
        assert_eq!(block.dimensions.y, 0.0);
        assert_eq!(block.dimensions.height, 50.0);
    }

    #[test]
    fn test_float_clear_left() {
        let (dom_tree, style_tree, mut text_context) = process_html_raw!("float_clear.html.zst", true);
        let layout = layout_from!(dom_tree, style_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let body = &root.children[0];
        assert_eq!(body.children.len(), 4);

        let float1 = &body.children[0];
        assert_eq!(float1.dimensions.x, 0.0);
        assert_eq!(float1.dimensions.y, 0.0);
        assert_eq!(float1.dimensions.height, 100.0);

        let clear_left = &body.children[1];
        assert_eq!(clear_left.dimensions.x, 0.0);
        assert_eq!(clear_left.dimensions.y, 100.0);
        assert_eq!(clear_left.dimensions.height, 50.0);

        let float2 = &body.children[2];
        assert_eq!(float2.dimensions.x, 0.0);
        assert_eq!(float2.dimensions.y, 150.0);
        assert_eq!(float2.dimensions.height, 100.0);

        let clear_both = &body.children[3];
        assert_eq!(clear_both.dimensions.x, 0.0);
        assert_eq!(clear_both.dimensions.y, 250.0);
        assert_eq!(clear_both.dimensions.height, 50.0);
    }

    #[test]
    fn test_float_clear_right() {
        let (dom_tree, style_tree, mut text_context) = process_html_raw!("float_clear_right.html.zst", true);
        let layout = layout_from!(dom_tree, style_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let body = &root.children[0];
        assert_eq!(body.children.len(), 3);

        let float_right = &body.children[0];
        assert_eq!(float_right.dimensions.x, 700.0);
        assert_eq!(float_right.dimensions.y, 0.0);
        assert_eq!(float_right.dimensions.height, 80.0);

        let block = &body.children[1];
        assert_eq!(block.dimensions.x, 0.0);
        assert_eq!(block.dimensions.y, 0.0);
        assert_eq!(block.dimensions.height, 30.0);

        let clear_right = &body.children[2];
        assert_eq!(clear_right.dimensions.x, 0.0);
        assert_eq!(clear_right.dimensions.y, 80.0);
        assert_eq!(clear_right.dimensions.height, 40.0);
    }
}
