use std::time::Instant;

use api::collector::DefaultCollector;
use html_parser::parser::streaming::HtmlStreamParser;
use network::web::client::WebClient;
use reqwest::Client;

#[tokio::main]
async fn main() {
    let content = fetch_from_url("https://docs.rs/").await;

    if let Err(e) = content {
        eprintln!("Error fetching file: {}", e);
        return;
    }
    let content = content.unwrap();

    let parser = HtmlStreamParser::builder(content.as_bytes())
        .collector(DefaultCollector::default())
        .build();

    let start_time = Instant::now();
    let parser_result = parser.parse();
    let elapsed_time = start_time.elapsed();
    println!("Parsing took: {:.2?}", elapsed_time);

    if let Err(e) = parser_result {
        eprintln!("Error parsing document: {}", e);
        return;
    }

    println!("Document parsed successfully.");
    let parse_result = parser_result.unwrap();
    let dom_tree = parse_result.dom_tree;

    std::fs::write(
        "resources/html/output/output.txt",
        format!("{:#?}", dom_tree),
    )
    .expect("Unable to write to file");
}

#[allow(dead_code)]
async fn fetch_from_url(url: &str) -> Result<String, String> {
    let client = Client::builder()
        .user_agent(format!(
            "browser-{}/{}-dev (testing; Rust 1.28.2; reqwest 0.12.18) andreeriksson444@gmail.com",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .expect("Failed to build HTTP client");

    let mut web_client = WebClient::builder(client).with_url(url).build();

    let result = web_client.setup_client().await;

    if let Err(e) = result {
        eprintln!("Error fetching URL: {}", e);
        return Err(e);
    }
    let response = result.unwrap();

    Ok(response)
}

#[allow(dead_code)]
fn fetch_from_file(html_file: &str) -> Result<String, std::io::Error> {
    use std::fs;

    let file = fs::read_to_string(format!("resources/html/{}.html", html_file));
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
