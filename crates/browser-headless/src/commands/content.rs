use crate::HeadlessEngine;

pub(crate) fn cmd_title(engine: &mut HeadlessEngine) -> Result<(), String> {
    if let Some(tab) = engine.browser.tab_manager().active_tab() {
        println!("{}", tab.page().title());
    } else {
        println!("No active tab");
    }
    Ok(())
}

pub(crate) fn cmd_url(engine: &mut HeadlessEngine) -> Result<(), String> {
    if let Some(tab) = engine.browser.tab_manager().active_tab() {
        if let Some(url) = tab.page().document_url() {
            println!("{}", url);
        } else {
            println!("about:blank");
        }
    } else {
        println!("No active tab");
    }
    Ok(())
}

pub(crate) fn cmd_headers(engine: &mut HeadlessEngine) -> Result<(), String> {
    for header in engine.browser.headers().iter() {
        println!("{}: {}", header.0, header.1.to_str().unwrap_or(""));
    }
    Ok(())
}

pub(crate) fn cmd_body(engine: &mut HeadlessEngine) -> Result<(), String> {
    if let Some(active_tab) = engine.browser.tab_manager().active_tab() {
        println!("{}", active_tab.page().document());
    } else {
        println!("No active tab.");
    }
    Ok(())
}

pub(crate) fn cmd_cookies(engine: &mut HeadlessEngine, domain: Option<&str>) -> Result<(), String> {
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

pub(crate) fn cmd_info(engine: &mut HeadlessEngine) -> Result<(), String> {
    if let Some(tab) = engine.browser.tab_manager().active_tab() {
        let page = tab.page();
        println!("Title: {}", page.title());
        println!(
            "URL: {}",
            page.document_url()
                .map(|u| u.as_str())
                .unwrap_or("about:blank")
        );
        println!("Viewport: {}x{}", engine.viewport_width, engine.viewport_height);

        if let Some(ref layout) = engine.layout_tree {
            println!("Content size: {}x{}", layout.content_width, layout.content_height);
            println!("Root nodes: {}", layout.root_nodes.len());
        } else {
            println!("Layout: not computed");
        }
    } else {
        println!("No active tab");
    }
    Ok(())
}
