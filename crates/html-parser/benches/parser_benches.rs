use std::fs::File;
use std::io::Read;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};
use tracing::{error, info};

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
                let mut parser = HtmlStreamParser::simple(std::io::Cursor::new(html_content.clone()));
                let mut success = false;

                loop {
                    let step = parser.step();

                    if let Err(e) = step {
                        error!("Error during parsing step: {}", e);
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
                                    error!("Error extracting script content: {}", e);
                                    break;
                                }

                                info!("Extracted Script Content: {}", script_content.unwrap());
                                let _ = parser.resume();
                            }
                            BlockedReason::WaitingForStyle(_) => {
                                let style_content = parser.extract_style_content();

                                if let Err(e) = style_content {
                                    error!("Error extracting style content: {}", e);
                                    break;
                                }

                                info!("Extracted Style Content: {}", style_content.unwrap());
                                let _ = parser.resume();
                            }
                            BlockedReason::SVGContent => {
                                let svg_content = parser.extract_svg_content();

                                if let Err(e) = svg_content {
                                    error!("Error extracting SVG content: {}", e);
                                    break;
                                }

                                info!("Extracted SVG Content: {}", svg_content.unwrap());
                                let _ = parser.resume();
                            }
                            _ => {
                                error!("Parser blocked for unhandled reason: {:?}", reason);
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
