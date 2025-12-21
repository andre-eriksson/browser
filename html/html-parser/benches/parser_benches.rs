use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::parser::streaming::HtmlStreamParser;
use html_syntax::collector::DefaultCollector;
use parsing::{ScriptHandler, StyleHandler};

struct JsParserImpl;
impl ScriptHandler for JsParserImpl {
    fn process_js(&mut self, _js_char: char) {}
}

struct CSSParserImpl;
impl StyleHandler for CSSParserImpl {
    fn process_css(&mut self, _css_char: char) {}
}

fn criterion_benchmark(c: &mut Criterion) {
    let prefix_path = "./benches/resources/";
    let files = ["deep.html", "flat.html", "mixed.html"];

    for &file in &files {
        let html_content = std::fs::read_to_string(format!("{}{}", prefix_path, file))
            .unwrap_or_else(|_| panic!("Failed to read file: {}{}", prefix_path, file));

        c.bench_function(&format!("streaming_html_parse_{}", file), |b| {
            b.iter(|| {
                let style_handler = Box::new(CSSParserImpl);
                let script_handler = Box::new(JsParserImpl);
                let parser = HtmlStreamParser::new(
                    std::io::Cursor::new(html_content.clone()),
                    Some(1024 * 8),
                    style_handler,
                    script_handler,
                );
                let parsing_result = parser.parse(Some(DefaultCollector::default()));
                assert!(parsing_result.is_ok(), "Parsing failed");
            });
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
