use std::fs::File;
use std::io::Read;
use std::time::Duration;

use criterion::{Criterion, criterion_group, criterion_main};
use html_parser::{BlockedReason, HtmlStreamParser, ParserState, Script};
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

                let result = loop {
                    let Ok(state) = parser.step() else {
                        error!("Error during parsing step: {}", parser.step().err().unwrap());
                        break None;
                    };

                    match state {
                        ParserState::Running => continue,
                        ParserState::Completed(result) => {
                            break Some(result);
                        }
                        ParserState::Blocked(reason) => match reason {
                            BlockedReason::WaitingForScript { script, .. } => match script {
                                Script::External { src, .. } => {
                                    info!("Parser blocked waiting for external script: src={}", src);
                                }
                                Script::Inline {
                                    data,
                                    type_attr: mime_type,
                                } => {
                                    let Ok(script_content) = data else {
                                        error!("Error extracting inline script content: {}", data.err().unwrap());
                                        break None;
                                    };

                                    info!(
                                        "Extracted Inline Script Content (mime_type={}): {}",
                                        mime_type, script_content
                                    );
                                }
                            },
                            BlockedReason::WaitingForStyle {
                                data,
                                attributes: _,
                            } => {
                                let Ok(css) = data else {
                                    error!("Error extracting style content: {}", data.err().unwrap());
                                    break None;
                                };

                                info!("Extracted Style Content: {}", css);
                            }
                            BlockedReason::SVGContent { data } => {
                                let Ok(svg) = data else {
                                    error!("Error extracting SVG content: {}", data.err().unwrap());
                                    break None;
                                };

                                info!("Extracted SVG Content: {}", svg);
                            }
                            _ => {
                                error!("Parser blocked for unhandled reason: {:?}", reason);
                                break None;
                            }
                        },
                    }
                };

                assert!(result.is_some(), "HTML parsing did not complete successfully.");
            });
        });
    }
}

criterion_group!(name = benches; config = criterion_config(); targets = criterion_benchmark);
criterion_main!(benches);
