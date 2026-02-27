//! Benchmarks for the CSS CSSOM
//!
//! Run with: cargo bench -p css-cssom
//!
//! These benchmarks measure CSSOM construction performance separately from
//! tokenization and parsing by pre-parsing the CSS input before benchmarking.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use css_cssom::{CSSRule, CSSStyleRule, CSSStyleSheet, StylesheetOrigin};
use css_parser::{CssParser, KnownProperty, Property, Stylesheet};
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

/// CSS with many declarations for property access benchmarks
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

/// CSS with multiple style rules for iteration benchmarks
const MULTIPLE_RULES_CSS: &str = r#"
.rule1 { color: red; }
.rule2 { color: blue; }
.rule3 { color: green; }
.rule4 { color: yellow; }
.rule5 { color: purple; }
.rule6 { color: orange; }
.rule7 { color: pink; }
.rule8 { color: cyan; }
.rule9 { color: magenta; }
.rule10 { color: lime; }
.rule11 { margin: 10px; }
.rule12 { margin: 20px; }
.rule13 { margin: 30px; }
.rule14 { margin: 40px; }
.rule15 { margin: 50px; }
.rule16 { padding: 10px; }
.rule17 { padding: 20px; }
.rule18 { padding: 30px; }
.rule19 { padding: 40px; }
.rule20 { padding: 50px; }
"#;

/// Pre-parse CSS to get a Stylesheet for benchmarking CSSOM construction
fn parse(css: &str) -> Stylesheet {
    let mut parser = CssParser::default();
    parser.parse_css(css, true)
}

fn bench_from_stylesheet_simple(c: &mut Criterion) {
    let parsed = parse(SIMPLE_CSS);

    c.bench_function("cssom_from_stylesheet_simple", |b| {
        b.iter(|| {
            let stylesheet: CSSStyleSheet = black_box(parsed.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_from_stylesheet_medium(c: &mut Criterion) {
    let parsed = parse(MEDIUM_CSS);

    c.bench_function("cssom_from_stylesheet_medium", |b| {
        b.iter(|| {
            let stylesheet: CSSStyleSheet = black_box(parsed.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_from_stylesheet_complex(c: &mut Criterion) {
    let parsed = parse(COMPLEX_CSS);

    c.bench_function("cssom_from_stylesheet_complex", |b| {
        b.iter(|| {
            let stylesheet: CSSStyleSheet = black_box(parsed.clone()).into();
            black_box(stylesheet)
        })
    });
}

fn bench_cssom_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("cssom_throughput");

    let inputs: [(&str, &str); 5] = [
        ("simple", SIMPLE_CSS),
        ("medium", MEDIUM_CSS),
        ("complex", COMPLEX_CSS),
        ("declarations", DECLARATION_HEAVY_CSS),
        ("multiple_rules", MULTIPLE_RULES_CSS),
    ];

    for (name, css) in inputs {
        let parsed = parse(css);
        // Measure throughput based on rule count (more relevant for CSSOM)
        group.throughput(Throughput::Elements(parsed.rules.len() as u64));
        group.bench_with_input(BenchmarkId::new("rules", name), &parsed, |b, parsed| {
            b.iter(|| {
                let stylesheet: CSSStyleSheet = black_box(parsed.clone()).into();
                black_box(stylesheet)
            })
        });
    }

    group.finish();
}

fn bench_cssom_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("cssom_scaling");

    // Generate CSS of different sizes by repeating the medium CSS
    let sizes = [1, 10, 50, 100];

    for size in sizes {
        let css = MEDIUM_CSS.repeat(size);
        let parsed = parse(&css);
        group.throughput(Throughput::Elements(parsed.rules.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("rules", size * 4), // ~4 rules per repetition
            &parsed,
            |b, parsed| {
                b.iter(|| {
                    let stylesheet: CSSStyleSheet = black_box(parsed.clone()).into();
                    black_box(stylesheet)
                })
            },
        );
    }

    group.finish();
}

fn bench_get_style_rules(c: &mut Criterion) {
    // For this benchmark, we use the full CSSStyleSheet since we're
    // benchmarking the get_style_rules method, not construction
    let stylesheet = CSSStyleSheet::from_css(COMPLEX_CSS, StylesheetOrigin::Author, true);

    c.bench_function("cssom_get_style_rules", |b| {
        b.iter(|| {
            let style_rules = stylesheet.get_style_rules();
            black_box(style_rules)
        })
    });
}

fn bench_css_rules_access(c: &mut Criterion) {
    let stylesheet = CSSStyleSheet::from_css(MULTIPLE_RULES_CSS, StylesheetOrigin::Author, true);

    c.bench_function("cssom_css_rules_access", |b| {
        b.iter(|| {
            let rules = stylesheet.css_rules();
            black_box(rules.len())
        })
    });
}

fn bench_to_css_string(c: &mut Criterion) {
    let mut group = c.benchmark_group("cssom_serialization");

    let inputs: [(&str, &str); 3] = [
        ("simple", SIMPLE_CSS),
        ("medium", MEDIUM_CSS),
        ("complex", COMPLEX_CSS),
    ];

    for (name, css) in inputs {
        let stylesheet = CSSStyleSheet::from_css(css, StylesheetOrigin::Author, true);
        group.bench_with_input(BenchmarkId::from_parameter(name), &stylesheet, |b, sheet| {
            b.iter(|| {
                let css_text = sheet.to_css_string();
                black_box(css_text)
            })
        });
    }

    group.finish();
}

fn bench_insert_delete_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("cssom_rule_manipulation");

    group.bench_function("insert_rule", |b| {
        b.iter_batched(
            || CSSStyleSheet::from_css(SIMPLE_CSS, StylesheetOrigin::Author, true),
            |mut stylesheet| {
                let new_rule = CSSRule::Style(CSSStyleRule::new("p".to_string()));
                stylesheet.insert_rule(new_rule, 0).unwrap();
                black_box(stylesheet)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.bench_function("delete_rule", |b| {
        b.iter_batched(
            || CSSStyleSheet::from_css(MULTIPLE_RULES_CSS, StylesheetOrigin::Author, true),
            |mut stylesheet| {
                stylesheet.delete_rule(0).unwrap();
                black_box(stylesheet)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_property_access(c: &mut Criterion) {
    let stylesheet = CSSStyleSheet::from_css(DECLARATION_HEAVY_CSS, StylesheetOrigin::Author, true);

    c.bench_function("cssom_property_access", |b| {
        b.iter(|| {
            let rules = stylesheet.css_rules();
            if let Some(CSSRule::Style(style_rule)) = rules.first() {
                let color = style_rule.get_property_value(Property::Known(KnownProperty::Color));
                let margin = style_rule.get_property_value(Property::Known(KnownProperty::MarginTop));
                let font = style_rule.get_property_value(Property::Known(KnownProperty::FontFamily));
                black_box((color, margin, font));
            }
        })
    });
}

fn bench_property_modification(c: &mut Criterion) {
    let mut group = c.benchmark_group("cssom_property_modification");

    group.bench_function("set_property", |b| {
        b.iter_batched(
            || CSSStyleRule::new("div".to_string()),
            |mut style_rule| {
                style_rule.set_property(Property::Known(KnownProperty::Color), "blue".to_string(), false);
                style_rule.set_property(Property::Known(KnownProperty::Margin), "10px".to_string(), false);
                style_rule.set_property(Property::Known(KnownProperty::Padding), "5px".to_string(), true);
                black_box(style_rule)
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_declaration_iteration(c: &mut Criterion) {
    let stylesheet = CSSStyleSheet::from_css(DECLARATION_HEAVY_CSS, StylesheetOrigin::Author, true);

    c.bench_function("cssom_declaration_iteration", |b| {
        b.iter(|| {
            let rules = stylesheet.css_rules();
            let mut count = 0;
            for rule in rules {
                if let CSSRule::Style(style_rule) = rule {
                    for decl in style_rule.declarations() {
                        black_box(decl.property());
                        black_box(decl.value());
                        count += 1;
                    }
                }
            }
            black_box(count)
        })
    });
}

criterion_group!(
    benches,
    bench_from_stylesheet_simple,
    bench_from_stylesheet_medium,
    bench_from_stylesheet_complex,
    bench_cssom_throughput,
    bench_cssom_scaling,
    bench_get_style_rules,
    bench_css_rules_access,
    bench_to_css_string,
    bench_insert_delete_rule,
    bench_property_access,
    bench_property_modification,
    bench_declaration_iteration,
);

criterion_main!(benches);
