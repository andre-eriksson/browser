use std::hint::black_box;

use api::collector::DefaultCollector;
use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::parser::streaming::HtmlStreamParser;

fn fetch_from_file(html_file: &str) -> Result<String, std::io::Error> {
    use std::path::Path;

    // Try different possible paths since benchmark working directory can vary
    let possible_paths = [
        format!("resources/html/{}.html", html_file),
        format!("../resources/html/{}.html", html_file),
        format!("../../resources/html/{}.html", html_file),
    ];

    let mut file_result = None;
    for path in &possible_paths {
        if Path::new(path).exists() {
            file_result = Some(std::fs::read_to_string(path));
            break;
        }
    }

    let file = match file_result {
        Some(result) => result,
        None => {
            eprintln!(
                "Could not find file {} in any of the expected locations",
                html_file
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File {}.html not found in resources/html/", html_file),
            ));
        }
    };

    if let Err(e) = file {
        eprintln!("Error reading file: {}", e);
        return Err(e);
    }
    let content = file.unwrap();

    if content.is_empty() {
        eprintln!("File is empty or not found: {}", html_file);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found or empty",
        ));
    }

    Ok(content)
}

pub fn parse_benchmark(c: &mut Criterion) {
    let content = fetch_from_file("insta");

    if let Err(e) = content {
        eprintln!("Error fetching file: {}", e);
        return;
    }
    let content = content.unwrap();

    c.bench_function("parse_html", |b| {
        b.iter(|| {
            let parser = HtmlStreamParser::new(content.as_bytes(), None);
            let parser_result = black_box(parser.parse::<DefaultCollector>(None));
            if let Err(e) = &parser_result {
                eprintln!("Error parsing document: {}", e);
            }
        });
    });
}

criterion_group!(benches, parse_benchmark);
criterion_main!(benches);
