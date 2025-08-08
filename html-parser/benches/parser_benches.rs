use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::{collector::DefaultCollector, parser::streaming::HtmlStreamParser};

fn criterion_benchmark(c: &mut Criterion) {
    let prefix_path = "../test-resources/html/large/";
    let files = [
        "Amazon.html",
        "HTML5Up.html",
        "Instagram.html",
        "Reuters.html",
        "Wikipedia.html",
        "Youtube.html",
    ];

    for &file in &files {
        let html_content = std::fs::read_to_string(format!("{}{}", prefix_path, file))
            .expect("Failed to read HTML file");

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
