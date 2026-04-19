use css_cssom::ComponentValue;
use css_selectors::{generate_selector_list, matches_compound};
use css_tokenizer::CssTokenizer;
use html_dom::{NodeData, NodeId};

use crate::HeadlessEngine;

pub fn cmd_dom(engine: &HeadlessEngine, selector: &str) -> Result<(), String> {
    let Some(page) = &engine.page else {
        return Err("No page loaded. Please navigate to a URL first.".to_string());
    };

    let document = page.document();

    let component_values: Vec<ComponentValue> = CssTokenizer::new(selector, false)
        .map(ComponentValue::Token)
        .collect();
    let selector_lists = generate_selector_list(&component_values);

    if selector_lists.is_empty() {
        return Err(format!("Invalid selector: {}", selector));
    }

    let mut matches_found: Vec<(NodeId, String)> = Vec::new();

    for node in &document.nodes {
        if let NodeData::Element(element) = &node.data {
            for selector_sequence in &selector_lists {
                if matches_compound(selector_sequence, document, node, element.class_set.as_ref()) {
                    let desc = describe_element(element, node.id);
                    matches_found.push((node.id, desc));
                    break;
                }
            }
        }
    }

    if matches_found.is_empty() {
        println!("No elements match selector: {}", selector);
    } else {
        println!("Found {} matching element(s):", matches_found.len());
        for (node_id, description) in &matches_found {
            println!("  [{}] {}", node_id.0, description);
        }
    }

    Ok(())
}

fn describe_element(element: &html_dom::Element, node_id: NodeId) -> String {
    let tag = element.tag_name();
    let id_str = element
        .id()
        .map(|id| format!("#{}", id))
        .unwrap_or_default();
    let classes: Vec<_> = element.classes().take(3).collect();
    let class_str = if classes.is_empty() {
        String::new()
    } else {
        format!(".{}", classes.join("."))
    };

    format!("<{}{}{}> (NodeId: {})", tag, id_str, class_str, node_id.0)
}
