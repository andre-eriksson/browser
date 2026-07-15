#[cfg(test)]
mod tests {
    use std::{
        fs::File,
        io::{BufReader, Cursor, Read},
    };

    use std::net::Ipv4Addr;
    use url::Url;

    use browser_preferences::theme::ThemeCategory;
    use browser_ui::load_fallback_fonts;
    use cosmic_text::FontSystem;
    use css_cssom::{CSSStyleSheet, StylesheetOrigin};
    use css_display::BoxTree;
    use css_style::{AbsoluteContext, StyleTree};
    use css_values::color::Color;
    use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
    use io::embedded::DEFAULT_CSS;
    use layout::{ImageContext, LayoutImage, LayoutInput, LayoutTree, NodeId, Rect, TextContext};

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
            let user_agent_css = DEFAULT_CSS.load();
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

            let result = loop {
                let Ok(state) = parser.step() else {
                    panic!("Parser error: {:?}", parser.step().err());
                };

                match state {
                    ParserState::Running => continue,
                    ParserState::Blocked(reason) => match reason {
                        BlockedReason::WaitingForStyle {
                            data,
                            attributes: _,
                        } => {
                            if let Ok(css) = data {
                                let stylesheet = CSSStyleSheet::from_css(&css, StylesheetOrigin::Author, true);
                                stylesheets.push(stylesheet);
                            }
                        }
                        _ => {
                            panic!("Test files will only block on styles.");
                        }
                    },
                    ParserState::Completed(result) => {
                        break result;
                    }
                }
            };

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

            let document = result.dom_tree;
            let style_tree = StyleTree::build(None, &absolute_ctx, &document, &stylesheets);
            let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
            let text_context = TextContext::new(font_system);
            (document, style_tree, text_context)
        }};
    }

    /// Runs layout from a pre-built `StyleTree`, optionally with an
    /// `ImageContext` for known image dimensions.
    macro_rules! layout_from {
        ($dom_tree:expr, $box_tree:expr, $text_context:expr) => {{
            let img_ctx = ImageContext::new();
            LayoutTree::compute_layout(
                &mut LayoutInput {
                    dom: &$dom_tree,
                    box_tree: &$box_tree,
                    text: $text_context,
                    image: &img_ctx,
                },
                viewport(),
            )
        }};
        ($dom_tree:expr, $box_tree:expr, $text_context:expr, $image_ctx:expr) => {{
            LayoutTree::compute_layout(
                &mut LayoutInput {
                    dom: &$dom_tree,
                    box_tree: &$box_tree,
                    text: $text_context,
                    image: &$image_ctx,
                },
                viewport(),
            )
        }};
    }

    /// Convenience: parse HTML and immediately compute layout (no image
    /// context).
    macro_rules! process_html {
        ($path:literal, $user_agent_css:expr) => {{
            let (dom_tree, style_tree, mut text_context) = process_html_raw!($path, $user_agent_css);
            let box_tree = BoxTree::new(&dom_tree, &style_tree);
            let img_ctx = ImageContext::new();
            LayoutTree::compute_layout(
                &mut LayoutInput {
                    dom: &dom_tree,
                    box_tree: &box_tree,
                    text: &mut text_context,
                    image: &img_ctx,
                },
                viewport(),
            )
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
        let root_node = &layout.nodes[root.index()].clone().unwrap();
        assert_eq!(root_node.dimensions.x, 0.0);
        assert_eq!(root_node.dimensions.y, 0.0);
        assert_eq!(root_node.dimensions.height, 400.0);
        assert_eq!(root_node.dimensions.height, layout.content_height);

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();
        assert_eq!(body_node.dimensions.x, 8.0);
        assert_eq!(body_node.dimensions.y, 20.0);
        assert_eq!(body_node.dimensions.height, 280.0);
        assert_eq!(body_node.dimensions.width, 784.0);
        assert_eq!(body_node.children.len(), 4);

        let first_div = &body_node.children[0];
        let first_div_node = &layout.nodes[first_div.index()].clone().unwrap();
        assert_eq!(first_div_node.dimensions.x, 28.0);
        assert_eq!(first_div_node.dimensions.y, 20.0);
        assert_eq!(first_div_node.dimensions.height, 30.0);
        assert_eq!(first_div_node.dimensions.width, 744.0);
        assert_eq!(first_div_node.margin.top.to_px(), 20.0);
        assert_eq!(first_div_node.margin.top, first_div_node.margin.bottom);

        let second_div = &body_node.children[1];
        let second_div_node = &layout.nodes[second_div.index()].clone().unwrap();
        assert_eq!(second_div_node.dimensions.x, 28.0);
        assert_eq!(second_div_node.dimensions.y, 70.0);
        assert_eq!(second_div_node.dimensions.height, 30.0);
        assert_eq!(second_div_node.dimensions.width, 724.0);
        assert_eq!(second_div_node.padding.top, 10.0);
        assert_eq!(second_div_node.padding.bottom, 10.0);
        assert_eq!(second_div_node.margin.top.to_px(), 20.0);
        assert_eq!(second_div_node.margin.top, first_div_node.margin.bottom);

        let third_div = &body_node.children[2];
        let third_div_node = &layout.nodes[third_div.index()].clone().unwrap();
        assert_eq!(third_div_node.dimensions.x, 28.0);
        assert_eq!(third_div_node.dimensions.y, 140.0);
        assert_eq!(third_div_node.dimensions.height, 30.0);
        assert_eq!(third_div_node.dimensions.width, 744.0);
        assert_eq!(third_div_node.margin.top.to_px(), 20.0);
        assert_eq!(third_div_node.margin.top, second_div_node.margin.bottom);

        let fourth_div = &body_node.children[3];
        let fourth_div_node = &layout.nodes[fourth_div.index()].clone().unwrap();
        assert_eq!(fourth_div_node.dimensions.x, 108.0);
        assert_eq!(fourth_div_node.dimensions.y, 270.0);
        assert_eq!(fourth_div_node.dimensions.height, 30.0);
        assert_eq!(fourth_div_node.dimensions.width, 584.0);
        assert_eq!(fourth_div_node.margin.top.to_px(), 100.0);
        assert_eq!(fourth_div_node.margin.top, fourth_div_node.margin.bottom);
    }

    #[test]
    fn test_collapsing_padding() {
        let layout = process_html!("collapsing_padding.html.zst", true);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();
        assert_eq!(root_node.dimensions.x, 0.0);
        assert_eq!(root_node.dimensions.y, 0.0);
        assert_eq!(root_node.dimensions.height, 436.0);
        assert_eq!(root_node.dimensions.height, layout.content_height);

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();
        assert_eq!(body_node.dimensions.x, 8.0);
        assert_eq!(body_node.dimensions.y, 8.0);
        assert_eq!(body_node.dimensions.height, 400.0);
        assert_eq!(body_node.dimensions.width, 764.0);
        assert_eq!(body_node.children.len(), 4);

        let first_div = &body_node.children[0];
        let first_div_node = &layout.nodes[first_div.index()].clone().unwrap();
        assert_eq!(first_div_node.dimensions.x, 38.0);
        assert_eq!(first_div_node.dimensions.y, 38.0);
        assert_eq!(first_div_node.dimensions.height, 30.0);
        assert_eq!(first_div_node.dimensions.width, 724.0);
        assert_eq!(first_div_node.margin.top.to_px(), 20.0);
        assert_eq!(first_div_node.margin.top, first_div_node.margin.bottom);

        let second_div = &body_node.children[1];
        let second_div_node = &layout.nodes[second_div.index()].clone().unwrap();
        assert_eq!(second_div_node.dimensions.x, 38.0);
        assert_eq!(second_div_node.dimensions.y, 88.0);
        assert_eq!(second_div_node.dimensions.height, 30.0);
        assert_eq!(second_div_node.dimensions.width, 704.0);
        assert_eq!(second_div_node.padding.top, 10.0);
        assert_eq!(second_div_node.padding.bottom, 10.0);
        assert_eq!(second_div_node.margin.top.to_px(), 20.0);
        assert_eq!(second_div_node.margin.top, first_div_node.margin.bottom);

        let third_div = &body_node.children[2];
        let third_div_node = &layout.nodes[third_div.index()].clone().unwrap();
        assert_eq!(third_div_node.dimensions.x, 38.0);
        assert_eq!(third_div_node.dimensions.y, 158.0);
        assert_eq!(third_div_node.dimensions.height, 30.0);
        assert_eq!(third_div_node.dimensions.width, 724.0);
        assert_eq!(third_div_node.margin.top.to_px(), 20.0);
        assert_eq!(third_div_node.margin.top, second_div_node.margin.bottom);

        let fourth_div = &body_node.children[3];
        let fourth_div_node = &layout.nodes[fourth_div.index()].clone().unwrap();
        assert_eq!(fourth_div_node.dimensions.x, 118.0);
        assert_eq!(fourth_div_node.dimensions.y, 288.0);
        assert_eq!(fourth_div_node.dimensions.height, 30.0);
        assert_eq!(fourth_div_node.dimensions.width, 564.0);
        assert_eq!(fourth_div_node.margin.top.to_px(), 100.0);
        assert_eq!(fourth_div_node.margin.top, fourth_div_node.margin.bottom);
    }

    #[test]
    fn test_mixed_content() {
        let layout = process_html!("mixed.html.zst", true);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();
        assert_eq!(root_node.dimensions.x, 0.0);
        assert_eq!(root_node.dimensions.y, 0.0);
        assert!(root_node.dimensions.height > 150.0 && root_node.dimensions.height < 200.0);
        assert_eq!(root_node.dimensions.height, layout.content_height);

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();
        assert_eq!(body_node.dimensions.x, 8.0);
        assert_eq!(body_node.dimensions.y, 8.0);
        assert!(body_node.dimensions.height > 130.0 && body_node.dimensions.height < 160.0);
        assert_eq!(body_node.children.len(), 4);

        let first_span = &body_node.children[0];
        let first_span_node = &layout.nodes[first_span.index()].clone().unwrap();
        assert_eq!(first_span_node.dimensions.x, 8.0);
        assert_eq!(first_span_node.dimensions.y, 8.0);
        assert_eq!(first_span_node.dimensions.height, 24.0);
        assert_eq!(first_span_node.dimensions.width, 784.0);

        let first_div = &body_node.children[1];
        let first_div_node = &layout.nodes[first_div.index()].clone().unwrap();
        assert_eq!(first_div_node.dimensions.x, 28.0);
        assert_eq!(first_div_node.dimensions.y, 52.0);
        assert_eq!(first_div_node.dimensions.height, 30.0);
        assert_eq!(first_div_node.dimensions.width, 744.0);

        // <br> is just adjusting the y position of the next element

        let second_div = &body_node.children[3];
        let second_div_node = &layout.nodes[second_div.index()].clone().unwrap();
        assert_eq!(second_div_node.dimensions.x, 28.0);
        assert!(second_div_node.dimensions.y > 110.0 && second_div_node.dimensions.y < 130.0);
        assert_eq!(second_div_node.dimensions.height, 30.0);
        assert_eq!(second_div_node.dimensions.width, 744.0);
    }

    #[test]
    fn test_child_calc_percentage_resolves_against_parent_size() {
        let (dom, style_tree, mut text_ctx) = process_html_raw!("calc_percent_child.html.zst", true);
        let box_tree = BoxTree::new(&dom, &style_tree);
        let layout = layout_from!(dom, box_tree, &mut text_ctx);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        let first_div = &body_node.children[0];
        let first_div_node = &layout.nodes[first_div.index()].clone().unwrap();

        let child = &first_div_node.children[0];
        let child_node = &layout.nodes[child.index()].clone().unwrap();

        assert_eq!(child_node.dimensions.width, 260.0);
        assert_eq!(child_node.dimensions.height, 260.0);
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
        let box_tree = BoxTree::new(&dom, &style_tree);
        let mut layout = layout_from!(dom, box_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        let container = &body_node.children[0];
        let container_node = &layout.nodes[container.index()].clone().unwrap();

        let img_parent = &container_node.children[1];
        let img_parent_node = &layout.nodes[img_parent.index()].clone().unwrap();

        let img = img_parent_node
            .children
            .iter()
            .find(|n| {
                let node = &layout.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .expect("should have an image node");

        let img_node = &layout.nodes[img.index()].clone().unwrap();

        assert_eq!(img_node.dimensions.width, 300.0);
        assert_eq!(img_node.dimensions.height, 150.0);
        assert!(
            img_node
                .image_data
                .as_ref()
                .unwrap()
                .image_needs_intrinsic_size
        );

        let after_p_y_before = container_node
            .children
            .last()
            .expect("container should have children");

        let after_p_y_before = layout.nodes[after_p_y_before.index()]
            .clone()
            .unwrap()
            .dimensions
            .y;
        let content_height_before = layout.content_height;

        let mut image_ctx = ImageContext::new();
        image_ctx.insert(
            img_node.node_id.unwrap(),
            LayoutImage {
                width: 640,
                height: 480,
                rgba: vec![],
            }
            .into(),
        );

        LayoutTree::relayout_node(
            &img_node.node_id.unwrap(),
            viewport(),
            &mut layout,
            &mut LayoutInput {
                dom: &dom,
                box_tree: &box_tree,
                text: &mut text_context,
                image: &image_ctx,
            },
        );

        let root2 = &layout.root_nodes[0];
        let root_node2 = &layout.nodes[root2.index()].clone().unwrap();

        let body2 = &root_node2.children[0];
        let body_node2 = &layout.nodes[body2.index()].clone().unwrap();

        let container2 = &body_node2.children[0];
        let container_node2 = &layout.nodes[container2.index()].clone().unwrap();

        let img_parent2 = &container_node2.children[1];
        let img_parent_node2 = &layout.nodes[img_parent2.index()].clone().unwrap();

        let img2 = img_parent_node2
            .children
            .iter()
            .find(|n| {
                let node = &layout.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .expect("should still have an image node");

        let img_node2 = &layout.nodes[img2.index()].clone().unwrap();

        assert_eq!(img_node2.dimensions.width, 640.0);
        assert_eq!(img_node2.dimensions.height, 480.0);
        assert!(
            !img_node2
                .image_data
                .as_ref()
                .unwrap()
                .image_needs_intrinsic_size
        );

        let after_p_y_after = container_node2
            .children
            .last()
            .expect("container should have children");

        let after_p_y_after = layout.nodes[after_p_y_after.index()]
            .clone()
            .unwrap()
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
        let box_tree = BoxTree::new(&dom, &style_tree);
        let mut layout = layout_from!(dom, box_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        let container = &body_node.children[0];
        let container_node = &layout.nodes[container.index()].clone().unwrap();

        let img_parent = &container_node.children[1];
        let img_parent_node = &layout.nodes[img_parent.index()].clone().unwrap();

        let img = img_parent_node
            .children
            .iter()
            .find(|n| {
                let node = &layout.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .expect("should have an image node");
        let img_node = &layout.nodes[img.index()].clone().unwrap();

        let mut image_ctx = ImageContext::new();
        image_ctx.insert(
            img_node.node_id.unwrap(),
            LayoutImage {
                width: 640,
                height: 480,
                rgba: vec![],
            }
            .into(),
        );

        LayoutTree::relayout_node(
            &img_node.node_id.unwrap(),
            viewport(),
            &mut layout,
            &mut LayoutInput {
                dom: &dom,
                box_tree: &box_tree,
                text: &mut text_context,
                image: &image_ctx,
            },
        );

        let root2 = &layout.root_nodes[0];
        let root_node2 = &layout.nodes[root2.index()].clone().unwrap();

        let body2 = &root_node2.children[0];
        let body_node2 = &layout.nodes[body2.index()].clone().unwrap();

        let container2 = &body_node2.children[0];
        let container_node2 = &layout.nodes[container2.index()].clone().unwrap();

        let img_parent2 = &container_node2.children[1];
        let img_parent_node2 = &layout.nodes[img_parent2.index()].clone().unwrap();

        let img2 = img_parent_node2
            .children
            .iter()
            .find(|n| {
                let node = &layout.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .expect("should still have an image node");
        let img_node2 = &layout.nodes[img2.index()].clone().unwrap();

        assert_eq!(img_node2.dimensions.width, container_node2.dimensions.width);
        assert_eq!(img_node2.dimensions.width, 784.0);
        assert_eq!(img_node2.dimensions.height, 588.0);
    }

    /// Verifies that relayout with the same `ImageContext` is idempotent –
    /// running it twice produces identical results.
    #[test]
    fn test_image_relayout_is_idempotent() {
        let (dom, style_tree, mut text_context) = process_html_raw!("image_relayout.html.zst", true);
        let box_tree = BoxTree::new(&dom, &style_tree);
        let mut image_ctx = ImageContext::new();
        image_ctx.insert(
            NodeId(14),
            LayoutImage {
                width: 640,
                height: 480,
                rgba: vec![],
            }
            .into(),
        );

        let layout_a = layout_from!(dom, box_tree, &mut text_context, &image_ctx);
        let layout_b = layout_from!(dom, box_tree, &mut text_context, &image_ctx);

        assert_eq!(layout_a.content_height, layout_b.content_height);

        let img_parent_a_node = &layout_a.nodes[layout_a.nodes[layout_a.nodes[layout_a.root_nodes[0].index()]
            .clone()
            .unwrap()
            .children[0]
            .index()]
        .clone()
        .unwrap()
        .children[0]
            .index()]
        .clone()
        .unwrap();

        let img_a_id = img_parent_a_node.children[1];
        let img_a_node = &layout_a.nodes[img_a_id.index()].clone().unwrap();

        let img_a = img_a_node
            .children
            .iter()
            .find(|n| {
                let node = &layout_a.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .unwrap();
        let img_a_node = &layout_a.nodes[img_a.index()].clone().unwrap();

        let img_parent_b_node = &layout_b.nodes[layout_b.nodes[layout_b.nodes[layout_b.root_nodes[0].index()]
            .clone()
            .unwrap()
            .children[0]
            .index()]
        .clone()
        .unwrap()
        .children[0]
            .index()]
        .clone()
        .unwrap();

        let img_b_id = img_parent_b_node.children[1];
        let img_b_node = &layout_a.nodes[img_b_id.index()].clone().unwrap();

        let img_b = img_b_node
            .children
            .iter()
            .find(|n| {
                let node = &layout_b.nodes[n.index()].clone().unwrap();
                node.image_data.is_some()
            })
            .unwrap();
        let img_b_node = &layout_b.nodes[img_b.index()].clone().unwrap();

        assert_eq!(img_a_node.dimensions.width, img_b_node.dimensions.width);
        assert_eq!(img_a_node.dimensions.height, img_b_node.dimensions.height);
        assert_eq!(img_a_node.dimensions.x, img_b_node.dimensions.x);
        assert_eq!(img_a_node.dimensions.y, img_b_node.dimensions.y);
    }

    #[test]
    fn test_float_basic() {
        let (dom, style_tree, mut text_context) = process_html_raw!("float_basic.html.zst", true);
        let box_tree = BoxTree::new(&dom, &style_tree);
        let layout = layout_from!(dom, box_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        assert_eq!(body_node.children.len(), 3);

        let float_left = &body_node.children[0];
        let float_left_node = &layout.nodes[float_left.index()].clone().unwrap();

        assert_eq!(float_left_node.dimensions.x, 0.0);
        assert_eq!(float_left_node.dimensions.y, 0.0);
        assert_eq!(float_left_node.dimensions.width, 100.0);
        assert_eq!(float_left_node.dimensions.height, 100.0);

        let float_right = &body_node.children[1];
        let float_right_node = &layout.nodes[float_right.index()].clone().unwrap();

        assert_eq!(float_right_node.dimensions.x, 700.0);
        assert_eq!(float_right_node.dimensions.y, 0.0);
        assert_eq!(float_right_node.dimensions.width, 100.0);
        assert_eq!(float_right_node.dimensions.height, 100.0);

        let block = &body_node.children[2];
        let block_node = &layout.nodes[block.index()].clone().unwrap();

        assert_eq!(block_node.dimensions.x, 0.0);
        assert_eq!(block_node.dimensions.y, 0.0);
        assert_eq!(block_node.dimensions.height, 50.0);
    }

    #[test]
    fn test_float_clear_left() {
        let (dom, style_tree, mut text_context) = process_html_raw!("float_clear.html.zst", true);
        let box_tree = BoxTree::new(&dom, &style_tree);
        let layout = layout_from!(dom, box_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        assert_eq!(body_node.children.len(), 4);

        let float1 = &body_node.children[0];
        let float1_node = &layout.nodes[float1.index()].clone().unwrap();

        assert_eq!(float1_node.dimensions.x, 0.0);
        assert_eq!(float1_node.dimensions.y, 0.0);
        assert_eq!(float1_node.dimensions.height, 100.0);

        let clear_left = &body_node.children[1];
        let clear_left_node = &layout.nodes[clear_left.index()].clone().unwrap();

        assert_eq!(clear_left_node.dimensions.x, 0.0);
        assert_eq!(clear_left_node.dimensions.y, 100.0);
        assert_eq!(clear_left_node.dimensions.height, 50.0);

        let float2 = &body_node.children[2];
        let float2_node = &layout.nodes[float2.index()].clone().unwrap();

        assert_eq!(float2_node.dimensions.x, 0.0);
        assert_eq!(float2_node.dimensions.y, 150.0);
        assert_eq!(float2_node.dimensions.height, 100.0);

        let clear_both = &body_node.children[3];
        let clear_both_node = &layout.nodes[clear_both.index()].clone().unwrap();

        assert_eq!(clear_both_node.dimensions.x, 0.0);
        assert_eq!(clear_both_node.dimensions.y, 250.0);
        assert_eq!(clear_both_node.dimensions.height, 50.0);
    }

    #[test]
    fn test_float_clear_right() {
        let (dom, style_tree, mut text_context) = process_html_raw!("float_clear_right.html.zst", true);
        let box_tree = BoxTree::new(&dom, &style_tree);
        let layout = layout_from!(dom, box_tree, &mut text_context);

        let root = &layout.root_nodes[0];
        let root_node = &layout.nodes[root.index()].clone().unwrap();

        let body = &root_node.children[0];
        let body_node = &layout.nodes[body.index()].clone().unwrap();

        assert_eq!(body_node.children.len(), 3);

        let float_right = &body_node.children[0];
        let float_right_node = &layout.nodes[float_right.index()].clone().unwrap();

        assert_eq!(float_right_node.dimensions.x, 700.0);
        assert_eq!(float_right_node.dimensions.y, 0.0);
        assert_eq!(float_right_node.dimensions.height, 80.0);

        let block = &body_node.children[1];
        let block_node = &layout.nodes[block.index()].clone().unwrap();

        assert_eq!(block_node.dimensions.x, 0.0);
        assert_eq!(block_node.dimensions.y, 0.0);
        assert_eq!(block_node.dimensions.height, 30.0);

        let clear_right = &body_node.children[2];
        let clear_right_node = &layout.nodes[clear_right.index()].clone().unwrap();

        assert_eq!(clear_right_node.dimensions.x, 0.0);
        assert_eq!(clear_right_node.dimensions.y, 80.0);
        assert_eq!(clear_right_node.dimensions.height, 40.0);
    }
}
