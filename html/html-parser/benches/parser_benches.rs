use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::parser::streaming::HtmlStreamParser;
use html_syntax::collector::DefaultCollector;

fn criterion_benchmark(c: &mut Criterion) {
    let prefix_path = "./benches/resources/";
    let files = ["deep.html", "flat.html", "mixed.html"];

    for &file in &files {
        let html_content = std::fs::read_to_string(format!("{}{}", prefix_path, file)).expect(
            format!(
                "Failed to read file: {}",
                format!("{}{}", prefix_path, file)
            )
            .as_str(),
        );

        c.bench_function(&format!("streaming_html_parse_{}", file), |b| {
            b.iter(|| {
                let parser = HtmlStreamParser::new(
                    std::io::Cursor::new(html_content.clone()),
                    Some(1024 * 8),
                );
                let parsing_result = parser.parse(Some(DefaultCollector::default()));
                assert!(parsing_result.is_ok(), "Parsing failed");
            });
        });
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
