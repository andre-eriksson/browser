use crate::HeadlessEngine;

pub fn cmd_title(engine: &HeadlessEngine) -> Result<(), String> {
    println!(
        "{}",
        engine
            .metadata
            .as_ref()
            .map(|m| m.title.trim())
            .unwrap_or("Untitled")
    );

    Ok(())
}

pub fn cmd_url(engine: &HeadlessEngine) -> Result<(), String> {
    if let Some(metadata) = &engine.metadata {
        println!("{}", metadata.url);
    } else {
        println!("about:blank");
    }
    Ok(())
}

pub fn cmd_headers(engine: &HeadlessEngine) -> Result<(), String> {
    for header in engine.browser.headers().iter() {
        println!("{}: {}", header.0, header.1.to_str().unwrap_or(""));
    }
    Ok(())
}

pub fn cmd_body(engine: &HeadlessEngine) -> Result<(), String> {
    println!(
        "{}",
        engine
            .page
            .as_ref()
            .map(|p| p.document().to_string())
            .unwrap_or_else(|| "No page loaded".to_string())
    );
    Ok(())
}

pub fn cmd_cookies(engine: &mut HeadlessEngine, domain: Option<&str>) -> Result<(), String> {
    let jar = engine.browser.cookie_jar().lock().unwrap();

    match domain {
        Some(domain) => {
            for cookie in jar.get_cookies_for_domain(domain) {
                println!("{}", cookie);
            }
        }
        None => {
            for cookie in jar.cookies() {
                println!("{}", cookie);
            }
        }
    }
    Ok(())
}

pub fn cmd_info(engine: &HeadlessEngine) -> Result<(), String> {
    let Some(metadata) = &engine.metadata else {
        println!("No metadata available");
        return Ok(());
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
    Ok(())
}
