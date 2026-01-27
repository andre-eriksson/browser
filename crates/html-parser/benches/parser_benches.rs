use criterion::{Criterion, criterion_group, criterion_main};
use html_dom::DefaultCollector;
use html_parser::{BlockedReason, HtmlStreamParser, ParserState};

fn criterion_benchmark(c: &mut Criterion) {
    let prefix_path = "./benches/resources/";
    let files = ["deep.html", "flat.html", "mixed.html"];

    for &file in &files {
        let html_content = std::fs::read_to_string(format!("{}{}", prefix_path, file))
            .unwrap_or_else(|_| panic!("Failed to read file: {}{}", prefix_path, file));

        c.bench_function(&format!("streaming_html_parse_{}", file), |b| {
            b.iter(|| {
                let mut parser = HtmlStreamParser::<_, DefaultCollector>::simple(
                    std::io::Cursor::new(html_content.clone()),
                );
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
                            BlockedReason::ParsingSVG => {
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

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
