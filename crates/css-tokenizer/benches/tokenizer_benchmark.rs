//! Benchmarks for the CSS tokenizer
//!
//! Run with: cargo bench -p css-tokenizer

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use css_tokenizer::CssTokenizer;
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

/// Complex CSS with various token types
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

/// CSS with lots of numbers and units
const NUMERIC_CSS: &str = r#"
.dimensions {
    width: 100px;
    height: 50.5em;
    margin: 10%;
    padding: 1.25rem;
    font-size: 16pt;
    line-height: 1.5;
    border-width: 0.5px;
    animation-duration: 300ms;
    transform: rotate(45deg) scale(1.2) translateX(-50%);
    opacity: 0.75;
}
"#;

/// CSS with many identifiers and selectors
const SELECTOR_HEAVY_CSS: &str = r#"
html body div.container section.content article.post header.post-header h1.post-title,
html body div.container section.content article.post header.post-header h2.post-subtitle,
html body div.container section.sidebar aside.widget ul.widget-list li.widget-item a.widget-link,
main > section:first-child > div:nth-child(2n+1) > p:not(.excluded):last-of-type {
    color: inherit;
    font-weight: normal;
}

#id1, #id2, #id3, #id4, #id5, #id6, #id7, #id8, #id9, #id10 {
    display: block;
}

.class1, .class2, .class3, .class4, .class5, .class6, .class7, .class8, .class9, .class10 {
    margin: 0;
}
"#;

fn bench_tokenize_simple(c: &mut Criterion) {
    c.bench_function("tokenize_simple", |b| {
        b.iter(|| {
            let tokens = CssTokenizer::tokenize(black_box(SIMPLE_CSS), true);
            black_box(tokens)
        })
    });
}

fn bench_tokenize_medium(c: &mut Criterion) {
    c.bench_function("tokenize_medium", |b| {
        b.iter(|| {
            let tokens = CssTokenizer::tokenize(black_box(MEDIUM_CSS), true);
            black_box(tokens)
        })
    });
}

fn bench_tokenize_complex(c: &mut Criterion) {
    c.bench_function("tokenize_complex", |b| {
        b.iter(|| {
            let tokens = CssTokenizer::tokenize(black_box(COMPLEX_CSS), true);
            black_box(tokens)
        })
    });
}

fn bench_tokenize_numeric_heavy(c: &mut Criterion) {
    c.bench_function("tokenize_numeric_heavy", |b| {
        b.iter(|| {
            let tokens = CssTokenizer::tokenize(black_box(NUMERIC_CSS), true);
            black_box(tokens)
        })
    });
}

fn bench_tokenize_selector_heavy(c: &mut Criterion) {
    c.bench_function("tokenize_selector_heavy", |b| {
        b.iter(|| {
            let tokens = CssTokenizer::tokenize(black_box(SELECTOR_HEAVY_CSS), true);
            black_box(tokens)
        })
    });
}

fn bench_tokenize_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_throughput");

    let inputs = [
        ("simple", SIMPLE_CSS),
        ("medium", MEDIUM_CSS),
        ("complex", COMPLEX_CSS),
        ("numeric", NUMERIC_CSS),
        ("selectors", SELECTOR_HEAVY_CSS),
    ];

    for (name, css) in inputs {
        group.throughput(Throughput::Bytes(css.len() as u64));
        group.bench_with_input(BenchmarkId::new("bytes", name), css, |b, input| {
            b.iter(|| {
                let tokens = CssTokenizer::tokenize(black_box(input), true);
                black_box(tokens)
            })
        });
    }

    group.finish();
}

fn bench_tokenize_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("tokenize_scaling");

    // Generate CSS of different sizes by repeating the medium CSS
    let sizes = [1, 10, 50, 100];

    for size in sizes {
        let css = MEDIUM_CSS.repeat(size);
        group.throughput(Throughput::Bytes(css.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("rules", size * 4), // ~4 rules per repetition
            &css,
            |b, input| {
                b.iter(|| {
                    let tokens = CssTokenizer::tokenize(black_box(input), true);
                    black_box(tokens)
                })
            },
        );
    }

    group.finish();
}

fn bench_specific_tokens(c: &mut Criterion) {
    let mut group = c.benchmark_group("specific_tokens");

    // Benchmark specific token types
    let inputs = [
        ("identifiers", "aaaa bbbb cccc dddd eeee ffff gggg hhhh iiii jjjj"),
        ("numbers", "123 456.789 0.5 100 200 300 400 500 600 700"),
        ("strings", r#""hello" 'world' "test" 'value' "string" 'text'"#),
        ("hashes", "#fff #000 #abc123 #header #footer #sidebar #main #content"),
        ("delimiters", "{ } [ ] ( ) : ; , . > + ~ * | ^"),
        ("at_keywords", "@media @import @keyframes @font-face @supports @charset"),
        ("functions", "rgb(1,2,3) rgba(1,2,3,4) calc(1+2) var(--x) url(test)"),
        ("whitespace", "    \n\n\n\t\t\t    \n  \t  \n  "),
        ("comments", "/* comment 1 */ /* comment 2 */ /* comment 3 */"),
    ];

    for (name, css) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(name), css, |b, input| {
            b.iter(|| {
                let tokens = CssTokenizer::tokenize(black_box(input), true);
                black_box(tokens)
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_tokenize_simple,
    bench_tokenize_medium,
    bench_tokenize_complex,
    bench_tokenize_numeric_heavy,
    bench_tokenize_selector_heavy,
    bench_tokenize_throughput,
    bench_tokenize_scaling,
    bench_specific_tokens,
);

criterion_main!(benches);
