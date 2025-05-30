use std::collections::HashMap;

use crate::patterns::patterns;

pub fn extract_attributes(tag_slice: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();

    let attr_regex = patterns::ATTR_REGEX
        .get()
        .expect("ATTR_REGEX not initialized");

    let bool_attr_regex = patterns::BOOL_ATTR_REGEX
        .get()
        .expect("BOOL_ATTR_REGEX not initialized");

    // Process attributes with values
    for cap in attr_regex.captures_iter(tag_slice) {
        let name = cap.get(1).unwrap().as_str().to_string();
        let value = cap
            .get(2)
            .or_else(|| cap.get(3))
            .or_else(|| cap.get(4))
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_string();

        attributes.insert(name, value);
    }

    // Process boolean attributes without values
    for cap in bool_attr_regex.captures_iter(tag_slice) {
        if let Some(name) = cap.get(1) {
            let attr_name = name.as_str().to_string();
            if !attributes.contains_key(&attr_name) {
                attributes.insert(attr_name, "".to_string());
            }
        }
    }

    attributes
}
