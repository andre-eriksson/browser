use std::fs::File;
use std::io::Read;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use html_dom::DefaultCollector;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};

fn load_fixture(path: &str) -> String {
    let file = File::open(path).expect("failed to open fixture");
    let mut decoder = zstd::Decoder::new(file).expect("failed to create decoder");

    let mut s = String::new();
    decoder
        .read_to_string(&mut s)
        .expect("failed to decompress");
    s
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(15))
        .sample_size(100)
}

fn criterion_benchmark(c: &mut Criterion) {
    let fixtures = [
        ("deep", load_fixture("benches/resources/deep.html.zst")),
        ("flat", load_fixture("benches/resources/flat.html.zst")),
        ("mixed", load_fixture("benches/resources/mixed.html.zst")),
    ];

    for (file, html_content) in fixtures.iter() {
        c.bench_function(&format!("streaming_html_parse_{}", file), |b| {
            b.iter(|| {
                let mut parser =
                    HtmlStreamParser::<_, DefaultCollector>::simple(std::io::Cursor::new(html_content.clone()));
                let mut success = false;

                loop {
                    let step = parser.step();

                    if let Err(e) = step {
                        eprintln!("Error during parsing step: {}", e);
                        break;
                    }

                    match parser.get_state() {
                        ParserState::Running => continue,
                        ParserState::Completed => {
                            success = true;
                            break;
                        }
                        ParserState::Blocked(reason) => match reason {
                            BlockedReason::WaitingForScript(_) => {
                                let script_content = parser.extract_script_content();

                                if let Err(e) = script_content {
                                    eprintln!("Error extracting script content: {}", e);
                                    break;
                                }

                                println!("Extracted Script Content: {}", script_content.unwrap());
                                let _ = parser.resume();
                            }
                            BlockedReason::WaitingForStyle(_) => {
                                let style_content = parser.extract_style_content();

                                if let Err(e) = style_content {
                                    eprintln!("Error extracting style content: {}", e);
                                    break;
                                }

                                println!("Extracted Style Content: {}", style_content.unwrap());
                                let _ = parser.resume();
                            }
                            BlockedReason::SVGContent => {
                                let svg_content = parser.extract_svg_content();

                                if let Err(e) = svg_content {
                                    eprintln!("Error extracting SVG content: {}", e);
                                    break;
                                }

                                println!("Extracted SVG Content: {}", svg_content.unwrap());
                                let _ = parser.resume();
                            }
                            _ => {
                                eprintln!("Parser blocked for unhandled reason: {:?}", reason);
                                break;
                            }
                        },
                    }
                }

                assert!(success, "HTML parsing did not complete successfully.");
            });
        });
    }
}

criterion_group!(name = benches; config = criterion_config(); targets = criterion_benchmark);
criterion_main!(benches);
