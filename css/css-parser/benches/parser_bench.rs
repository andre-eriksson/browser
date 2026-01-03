//! Benchmarks for the CSS parser
//!
//! Run with: cargo bench -p css-parser
//!
//! These benchmarks measure parsing performance separately from tokenization
//! by pre-tokenizing the CSS input before benchmarking.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use css_parser::Stylesheet;
use css_tokenizer::{CssToken, CssTokenizer};
use std::hint::black_box;

/// Simple CSS input
const SIMPLE_CSS: &str = "div { color: red; }";

/// Medium complexity CSS with multiple rules
const MEDIUM_CSS: &str = r#"
body {
    margin: 0;
    padding: 0;
    font-family: Arial, sans-serif;
}

.container {
    width: 100%;
    max-width: 1200px;
    margin: 0 auto;
}

#header {
    background-color: #333;
    color: white;
    padding: 20px;
}

a:hover {
    text-decoration: underline;
}
"#;

/// Complex CSS with various features
const COMPLEX_CSS: &str = r#"
@charset "UTF-8";
@import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;700&display=swap');

:root {
    --primary-color: #3498db;
    --secondary-color: rgba(52, 152, 219, 0.8);
    --spacing-unit: 8px;
}

@media screen and (min-width: 768px) {
    .container {
        width: calc(100% - 2 * var(--spacing-unit));
        padding: 1.5rem;
    }
}

@keyframes fadeIn {
    0% { opacity: 0; transform: translateY(-10px); }
    100% { opacity: 1; transform: translateY(0); }
}

.button {
    background: linear-gradient(135deg, var(--primary-color), var(--secondary-color));
    border-radius: 4px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
    transition: all 0.3s ease-in-out;
}

.button:hover,
.button:focus {
    transform: scale(1.05);
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
}

/* Multi-line comment
   spanning several lines
   with special characters: < > & " ' */
.special-chars {
    content: "Hello \"World\"";
    background-image: url(data:image/svg+xml,%3Csvg%3E%3C/svg%3E);
}

@font-face {
    font-family: 'CustomFont';
    src: url('font.woff2') format('woff2'),
         url('font.woff') format('woff');
    font-weight: 400;
    font-style: normal;
}

.grid-layout {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 1.5em;
}

[data-attribute="value"],
[data-attribute^="prefix"],
[data-attribute$="suffix"],
[data-attribute*="contains"] {
    color: inherit;
}

.pseudo-elements::before,
.pseudo-elements::after {
    content: '';
    display: block;
}

@supports (display: grid) {
    .modern-layout {
        display: grid;
    }
}
"#;

/// CSS with many declarations
const DECLARATION_HEAVY_CSS: &str = r#"
.styled-element {
    margin-top: 10px;
    margin-right: 20px;
    margin-bottom: 10px;
    margin-left: 20px;
    padding-top: 5px;
    padding-right: 10px;
    padding-bottom: 5px;
    padding-left: 10px;
    border-top: 1px solid #ccc;
    border-right: 1px solid #ccc;
    border-bottom: 1px solid #ccc;
    border-left: 1px solid #ccc;
    font-family: Arial, Helvetica, sans-serif;
    font-size: 16px;
    font-weight: 400;
    font-style: normal;
    line-height: 1.5;
    color: #333;
    background-color: #fff;
    background-image: none;
    background-repeat: no-repeat;
    background-position: center center;
    display: block;
    position: relative;
    top: 0;
    right: 0;
    bottom: 0;
    left: 0;
    width: 100%;
    height: auto;
    min-width: 0;
    max-width: none;
    min-height: 0;
    max-height: none;
    overflow: visible;
    z-index: 1;
    opacity: 1;
    visibility: visible;
    cursor: pointer;
    transform: none;
    transition: all 0.3s ease;
}
"#;

/// CSS with nested at-rules
const NESTED_AT_RULES_CSS: &str = r#"
@media screen {
    @supports (display: grid) {
        .grid-container {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
        }
    }

    @supports (display: flex) {
        .flex-container {
            display: flex;
            flex-direction: row;
        }
    }
}

@media print {
    body {
        font-size: 12pt;
        color: black;
    }

    .no-print {
        display: none !important;
    }
}

@layer base {
    body {
        margin: 0;
        padding: 0;
    }
}

@layer components {
    .button {
        padding: 10px 20px;
        border-radius: 4px;
    }
}
"#;

/// Pre-tokenize CSS for benchmarking
fn tokenize(css: &str) -> Vec<CssToken> {
    CssTokenizer::tokenize(css)
}

fn bench_parse_simple(c: &mut Criterion) {
    let tokens = tokenize(SIMPLE_CSS);

    c.bench_function("parse_simple", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(tokens.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_parse_medium(c: &mut Criterion) {
    let tokens = tokenize(MEDIUM_CSS);

    c.bench_function("parse_medium", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(tokens.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_parse_complex(c: &mut Criterion) {
    let tokens = tokenize(COMPLEX_CSS);

    c.bench_function("parse_complex", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(tokens.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_parse_declaration_heavy(c: &mut Criterion) {
    let tokens = tokenize(DECLARATION_HEAVY_CSS);

    c.bench_function("parse_declaration_heavy", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(tokens.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_parse_nested_at_rules(c: &mut Criterion) {
    let tokens = tokenize(NESTED_AT_RULES_CSS);

    c.bench_function("parse_nested_at_rules", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(tokens.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_parse_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_throughput");

    let inputs: [(&str, &str); 5] = [
        ("simple", SIMPLE_CSS),
        ("medium", MEDIUM_CSS),
        ("complex", COMPLEX_CSS),
        ("declarations", DECLARATION_HEAVY_CSS),
        ("nested_at_rules", NESTED_AT_RULES_CSS),
    ];

    for (name, css) in inputs {
        let tokens = tokenize(css);
        // Measure throughput based on token count (more relevant for parser)
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(BenchmarkId::new("tokens", name), &tokens, |b, tokens| {
            b.iter(|| {
                let stylesheet: Stylesheet = black_box(tokens.clone()).into();
                black_box(stylesheet)
            })
        });
    }

    group.finish();
}

fn bench_parse_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_scaling");

    // Generate CSS of different sizes by repeating the medium CSS
    let sizes = [1, 10, 50, 100];

    for size in sizes {
        let css = MEDIUM_CSS.repeat(size);
        let tokens = tokenize(&css);
        group.throughput(Throughput::Elements(tokens.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("rules", size * 4), // ~4 rules per repetition
            &tokens,
            |b, tokens| {
                b.iter(|| {
                    let stylesheet: Stylesheet = black_box(tokens.clone()).into();
                    black_box(stylesheet)
                })
            },
        );
    }

    group.finish();
}

fn bench_rule_extraction(c: &mut Criterion) {
    let mut group = c.benchmark_group("rule_extraction");

    let medium_tokens = tokenize(MEDIUM_CSS);
    let declaration_tokens = tokenize(DECLARATION_HEAVY_CSS);

    group.bench_function("parse_and_count_rules", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(medium_tokens.clone()).into();
            black_box(stylesheet.rules.len())
        })
    });

    group.bench_function("parse_and_extract_declarations", |b| {
        b.iter(|| {
            let stylesheet: Stylesheet = black_box(declaration_tokens.clone()).into();
            let mut count = 0;
            for rule in &stylesheet.rules {
                if let css_parser::Rule::QualifiedRule(qr) = rule {
                    count += qr.parse_declarations().len();
                }
            }
            black_box(count)
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_parse_simple,
    bench_parse_medium,
    bench_parse_complex,
    bench_parse_declaration_heavy,
    bench_parse_nested_at_rules,
    bench_parse_throughput,
    bench_parse_scaling,
    bench_rule_extraction,
);

criterion_main!(benches);
