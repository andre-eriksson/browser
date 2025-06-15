use html_parser::parser::{options::ParserOptions, streaming::StreamingParser};
use network::page::Page;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::builder()
        .user_agent(format!(
            "{}/{}-dev (testing; Rust 1.28.2; reqwest 0.12.18) andreeriksson444@gmail.com",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
        ))
        .build()
        .expect("Failed to build HTTP client");

    let url = "https://www.medieteknik.com/en";
    let mut page = Page::new(client, url);

    let result = page.fetch().await;

    if let Err(e) = result {
        eprintln!("Error fetching URL, {}", e);
        return;
    }
    let response = result.unwrap();
    let content = response.body;

    let mut parser = StreamingParser::new_with_options(
        content.as_bytes(),
        None,
        ParserOptions {
            collect_external_resources: true,
            ..Default::default()
        },
    );
    let parser_result = parser.parse();

    if let Err(e) = parser_result {
        eprintln!("Error parsing document: {}", e);
        return;
    }

    println!("Document parsed successfully.");

    let result = parser_result.unwrap();
    let metadata = result.metadata;

    if let Some(metadata) = metadata {
        for resource in metadata.external_resources.unwrap_or_default() {
            let element = resource.0.as_str();
            let external_url = resource.1;

            for url in external_url {
                match page.get_resource(element, &url).await {
                    Ok(resource_response) => {
                        println!("==========================");

                        println!("Element: {}", element);
                        println!("URL: {}", url);
                        println!("Status: {}", resource_response.status);
                        println!("Size: {} bytes", resource_response.size);
                        //println!("Headers: {:?}", resource_response.headers);
                        println!("==========================\n");
                    }
                    Err(e) => {
                        eprintln!("Error fetching resource {}: {}", url, e);
                    }
                }
            }
        }
    } else {
        println!("No metadata available.");
    }
}
