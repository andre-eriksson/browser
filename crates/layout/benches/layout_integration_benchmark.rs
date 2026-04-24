//! Benchmarks for end-to-end layout integration.

use std::{
    env,
    fs::File,
    hint::black_box,
    io::{BufReader, Read},
    path::{Path, PathBuf},
    sync::OnceLock,
    time::Duration,
};

use browser_config::BrowserConfig;
use browser_preferences::theme::ThemeCategory;
use browser_ui::load_fallback_fonts;
use cosmic_text::FontSystem;
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use css_cssom::{CSSStyleSheet, StylesheetOrigin};
use css_style::{AbsoluteContext, StyleTree};
use css_values::color::Color;
use html_dom::DocumentRoot;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
use io::{Resource, embeded::DEFAULT_CSS};
use layout::{ImageContext, LayoutEngine, Rect, TextContext};
use url::Url;

const BENCH_HTML_ENV: &str = "LAYOUT_BENCH_HTML";
const DEFAULT_FIXTURE: &str = "benches/wikipedia_tropical_storm.html.zst";
const DEFAULT_DOCUMENT_URL: &str = "http://127.0.0.1";

static DOCUMENT_URL: OnceLock<Url> = OnceLock::new();

fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(30))
        .sample_size(20)
}

fn viewport() -> Rect {
    Rect::new(0.0, 0.0, 800.0, 600.0)
}

fn document_url() -> &'static Url {
    DOCUMENT_URL.get_or_init(|| Url::parse(DEFAULT_DOCUMENT_URL).expect("default URL must be valid"))
}

fn absolute_context(viewport: Rect) -> AbsoluteContext<'static> {
    AbsoluteContext {
        viewport_width: viewport.width,
        viewport_height: viewport.height,
        root_font_size: 16.0,
        root_line_height_multiplier: 1.2,
        document_url: document_url(),
        theme_category: ThemeCategory::Light,
        root_color: Color::BLACK,
    }
}

fn load_user_agent_stylesheet() -> CSSStyleSheet {
    let user_agent_css = Resource::load_embedded(DEFAULT_CSS);
    let css = std::str::from_utf8(&user_agent_css).expect("embedded user agent CSS must be UTF-8");
    CSSStyleSheet::from_css(css, StylesheetOrigin::UserAgent, true)
}

fn resolve_fixture_path() -> PathBuf {
    if let Ok(path) = env::var(BENCH_HTML_ENV) {
        let candidate = PathBuf::from(path);
        if candidate.is_absolute() {
            return candidate;
        }

        let from_cwd = env::current_dir()
            .expect("failed to read current working directory")
            .join(&candidate);
        if from_cwd.exists() {
            return from_cwd;
        }

        return Path::new(env!("CARGO_MANIFEST_DIR")).join(candidate);
    }

    Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_FIXTURE)
}

fn load_html_fixture(path: &Path) -> String {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("zst"))
    {
        let file = File::open(path).expect("failed to open compressed HTML fixture");
        let mut decoder = zstd::Decoder::new(file).expect("failed to create zstd decoder");
        let mut html = String::new();
        decoder
            .read_to_string(&mut html)
            .expect("failed to decompress HTML fixture");
        return html;
    }

    std::fs::read_to_string(path).expect("failed to read HTML fixture")
}

fn parse_html_and_collect_styles(
    html: &str,
    user_agent_stylesheet: &CSSStyleSheet,
) -> (DocumentRoot, Vec<CSSStyleSheet>) {
    let mut stylesheets = vec![user_agent_stylesheet.clone()];

    let cursor = std::io::Cursor::new(html.as_bytes());
    let reader = BufReader::new(cursor);
    let mut parser = HtmlStreamParser::simple(reader);

    loop {
        match parser.step() {
            Ok(_) => match parser.get_state() {
                ParserState::Running => continue,
                ParserState::Blocked(reason) => match reason {
                    BlockedReason::WaitingForStyle(_) => {
                        let css = parser
                            .extract_style_content()
                            .expect("failed to extract inline style content");
                        stylesheets.push(CSSStyleSheet::from_css(&css, StylesheetOrigin::Author, true));
                        parser
                            .resume()
                            .expect("failed to resume HTML parser after style");
                    }
                    BlockedReason::WaitingForScript(_) => {
                        parser
                            .extract_script_content()
                            .expect("failed to extract script content");
                        parser
                            .resume()
                            .expect("failed to resume HTML parser after script");
                    }
                    BlockedReason::WaitingForResource(_, _, _) => {
                        parser
                            .resume()
                            .expect("failed to resume HTML parser after resource");
                    }
                    BlockedReason::SVGContent => {
                        parser
                            .extract_svg_content()
                            .expect("failed to extract SVG content");
                        parser
                            .resume()
                            .expect("failed to resume HTML parser after SVG content");
                    }
                },
                ParserState::Completed => break,
            },
            Err(error) => panic!("benchmark fixture failed to parse: {error}"),
        }
    }

    let result = parser.finalize();
    (result.dom_tree, stylesheets)
}

fn build_style_tree(dom: &DocumentRoot, stylesheets: &[CSSStyleSheet], viewport: Rect) -> StyleTree {
    let absolute_ctx = absolute_context(viewport);
    let config = BrowserConfig::default();
    StyleTree::build(&config, &absolute_ctx, dom, stylesheets)
}

fn new_text_context() -> TextContext {
    let font_system = FontSystem::new_with_fonts(load_fallback_fonts());
    TextContext::new(font_system)
}

fn bench_layout_pipeline(c: &mut Criterion) {
    let fixture_path = resolve_fixture_path();
    let fixture_name = fixture_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("custom_fixture")
        .to_string();

    let html = load_html_fixture(&fixture_path);
    let viewport = viewport();
    let user_agent_stylesheet = load_user_agent_stylesheet();
    let image_ctx = ImageContext::new();

    let (dom, stylesheets) = parse_html_and_collect_styles(&html, &user_agent_stylesheet);
    let style_tree = build_style_tree(&dom, &stylesheets, viewport);

    c.bench_function(&format!("layout/parse_html/{fixture_name}"), |b| {
        b.iter(|| {
            let (dom, stylesheets) = parse_html_and_collect_styles(black_box(html.as_str()), &user_agent_stylesheet);
            black_box((dom.root_nodes.len(), stylesheets.len()));
        })
    });

    let mut style_group = c.benchmark_group(format!("layout/style_tree_build/{fixture_name}"));
    style_group.throughput(Throughput::Bytes(html.len() as u64));
    style_group.bench_with_input(
        BenchmarkId::new("bytes", html.len()),
        &(&dom, &stylesheets),
        |b, (dom, stylesheets)| {
            b.iter(|| {
                let style_tree = build_style_tree(black_box(dom), black_box(stylesheets), viewport);
                black_box(style_tree.root_nodes.len());
            })
        },
    );
    style_group.finish();

    let mut layout_text_context = new_text_context();
    let mut layout_group = c.benchmark_group(format!("layout/compute_layout/{fixture_name}"));
    layout_group.throughput(Throughput::Elements(style_tree.root_nodes.len() as u64));
    layout_group.bench_with_input(
        BenchmarkId::new("root_nodes", style_tree.root_nodes.len()),
        &style_tree,
        |b, style_tree| {
            b.iter(|| {
                let layout = LayoutEngine::compute_layout(
                    &dom,
                    black_box(style_tree),
                    viewport,
                    &mut layout_text_context,
                    &image_ctx,
                );
                black_box(layout.content_height);
            })
        },
    );
    layout_group.finish();

    let mut full_text_context = new_text_context();
    c.bench_function(&format!("layout/full_pipeline/{fixture_name}"), |b| {
        b.iter(|| {
            let (dom, stylesheets) = parse_html_and_collect_styles(black_box(html.as_str()), &user_agent_stylesheet);
            let style_tree = build_style_tree(&dom, &stylesheets, viewport);
            let layout = LayoutEngine::compute_layout(&dom, &style_tree, viewport, &mut full_text_context, &image_ctx);
            black_box(layout.content_height);
        })
    });
}

criterion_group!(name = benches; config = criterion_config(); targets = bench_layout_pipeline);
criterion_main!(benches);
