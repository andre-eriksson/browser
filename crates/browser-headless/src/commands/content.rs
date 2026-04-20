use crate::HeadlessEngine;

pub fn cmd_title(engine: &HeadlessEngine) {
    println!(
        "{}",
        engine
            .metadata
            .as_ref()
            .map_or("Untitled", |m| m.title.trim())
    );
}

pub fn cmd_url(engine: &HeadlessEngine) {
    if let Some(metadata) = &engine.metadata {
        println!("{}", metadata.url);
    } else {
        println!("about:blank");
    }
}

pub fn cmd_headers(engine: &HeadlessEngine) {
    for header in engine.browser.headers() {
        println!("{}: {}", header.0, header.1.to_str().unwrap_or(""));
    }
}

pub fn cmd_body(engine: &HeadlessEngine) {
    println!(
        "{}",
        engine
            .page
            .as_ref()
            .map_or_else(|| "No page loaded".to_string(), |p| p.document().to_string())
    );
}

pub fn cmd_cookies(engine: &mut HeadlessEngine, domain: Option<&str>) {
    let jar = engine.browser.cookie_jar().lock().unwrap();

    match domain {
        Some(domain) => {
            for cookie in jar.get_cookies_for_domain(domain) {
                println!("{cookie}");
            }
        }
        None => {
            for cookie in jar.cookies() {
                println!("{cookie}");
            }
        }
    }
}

pub fn cmd_info(engine: &HeadlessEngine) {
    let Some(metadata) = &engine.metadata else {
        println!("No metadata available");
        return;
    };

    println!("Title: {}", metadata.title);
    println!("URL: {}", metadata.url.as_str());
    println!("Viewport: {}x{}", engine.viewport_width, engine.viewport_height);

    if let Some(ref layout) = engine.layout_tree {
        println!("Content size: {}x{}", layout.content_width, layout.content_height);
        println!("Root nodes: {}", layout.root_nodes.len());
    } else {
        println!("Layout: not computed");
    }
}
