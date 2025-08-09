# Rust powered Browser

A browser (kinda) project to learn Rust, creating and displaying HTML content via a UI, mostly from scratch. Perhaps I can use CSS and hopefully JS (in the future 😶). Not aiming to be fully HTML5 spec compliant just enough to display simple content.

## Preview

<img src="./docs/preview.webp" alt="A screenshot of 'https://developer.mozilla.org/en-US/' from the current build alongside YouTube and Amazon tabs">

## Features
- [ ] Engine
  - [ ] Delegate tasks
  - [x] Multithreaded?
  - [ ] Event queue
- [x] HTML Parser
  - [x] Basic DOM-tree builder
  - [x] Handling malformed tags
  - [x] Parses a decent amount of .html files correctly
  - [ ] Handling of more exceptions
- [ ] CSS Parser
  - [ ] Basic CSSOM builder
  - [ ] Cascading support
- [x] UI rendered
  - [x] Rendering the DOM-tree
  - [x] Applying basic CSS
  - [x] Drawing images
- [ ] Networking
  - [x] Able to fetch a single .html file from a URL
  - [x] Handling of Content-Security-Policy header
  - [ ] Handle POST, PUT, PATCH, DELETE, and other methods

## How to Run
1. Install Rust (https://www.rust-lang.org/tools/install)
2. Run
```sh
> cargo run
```

## Benchmarks (Criterion)

Date Captured: 2025-08-09 <sub>(YYYY-MM-DD)</sub>

### HTML Parser
**Amazon (734 KB (752 385 bytes))**

| /             | Lower bound | Estimate  | Upper bound |
|---------------|-------------|-----------|-------------|
| **R²**        | 0.0000013   | 0.0000013 | 0.0000013   |
| **Mean**      | 7.6968 ms   | 7.8430 ms | 8.0031 ms   |
| **Std. Dev.** | 575.54 µs   | 780.19 µs | 952.29 µs   |
| **Median**    | 7.5085 ms   | 7.5825 ms | 7.7762 ms   |
| **MAD**       | 398.16 µs   | 512.67 µs | 651.73 µs   |

---
**Instagram (1.13 MB (1 192 811 bytes))**

| /             | Lower bound | Estimate  | Upper bound |
|---------------|-------------|-----------|-------------|
| **R²**        | 0.0097040   | 0.0100184 | 0.0096019   |
| **Mean**      | 9.8219 ms   | 10.018 ms | 10.244 ms   |
| **Std. Dev.** | 684.20 µs   | 1.0868 ms | 1.4249 ms   |
| **Median**    | 9.5599 ms   | 9.6500 ms | 9.7912 ms   |
| **MAD**       | 210.88 µs   | 314.58 µs | 456.31 µs   |

---
**Reuters (1.17 MB (1 228 035 bytes))**

| /             | Lower bound | Estimate  | Upper bound |
|---------------|-------------|-----------|-------------|
| **R²**        | 0.0020448   | 0.0020892 | 0.0019837   |
| **Mean**      | 8.7646 ms   | 8.8209 ms | 8.9091 ms   |
| **Std. Dev.** | 108.32 µs   | 383.84 µs | 631.98 µs   |
| **Median**    | 8.7300 ms   | 8.7567 ms | 8.7801 ms   |
| **MAD**       | 79.690 µs   | 115.09 µs | 144.94 µs   |

---
**Wikipedia (1.17 MB (1 228 035 bytes))**

| /             | Lower bound | Estimate  | Upper bound |
|---------------|-------------|-----------|-------------|
| **R²**        | 0.0136386   | 0.0141083 | 0.0135466   |
| **Mean**      | 28.751 ms   | 29.278 ms | 29.856 ms   |
| **Std. Dev.** | 2.0208 ms   | 2.8357 ms | 3.5682 ms   |
| **Median**    | 28.322 ms   | 28.882 ms | 29.221 ms   |
| **MAD**       | 1.7147 ms   | 2.2238 ms | 2.5827 ms   |

---
**YouTube (1.17 MB (1 228 035 bytes))**

| /             | Lower bound | Estimate  | Upper bound |
|---------------|-------------|-----------|-------------|
| **R²**        | 0.0030197   | 0.0031241 | 0.0029967   |
| **Mean**      | 8.1355 ms   | 8.2936 ms | 8.4689 ms   |
| **Std. Dev.** | 637.81 µs   | 853.02 µs | 1.0246 ms   |
| **Median**    | 7.8552 ms   | 7.8861 ms | 7.9764 ms   |
| **MAD**       | 119.71 µs   | 175.46 µs | 325.82 µs   |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
