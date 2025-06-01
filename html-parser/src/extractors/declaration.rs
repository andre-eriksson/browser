use shared_types::dom;

use crate::patterns::Patterns;

pub fn extract_doctype_declaration(tag_slice: &str) -> dom::DoctypeDeclaration {
    let doctype_regex = Patterns::doctype_regex();

    if let Some(caps) = doctype_regex.captures(tag_slice) {
        let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let public_id = caps.get(2).map(|m| m.as_str().to_string());
        let system_id = caps.get(3).map(|m| m.as_str().to_string());

        dom::DoctypeDeclaration {
            name,
            public_id,
            system_id,
        }
    } else {
        dom::DoctypeDeclaration {
            name: String::new(),
            public_id: None,
            system_id: None,
        }
    }
}

pub fn extract_xml_declaration(tag_slice: &str) -> dom::XmlDeclaration {
    let xml_regex = Patterns::xml_declaration_regex();
    if let Some(caps) = xml_regex.captures(tag_slice) {
        let version = caps.get(1).map_or("", |m| m.as_str()).to_string();
        let encoding = caps.get(2).map(|m| m.as_str().to_string());
        let standalone = caps.get(3).map(|m| m.as_str() == "yes");

        dom::XmlDeclaration {
            version,
            encoding,
            standalone,
        }
    } else {
        dom::XmlDeclaration {
            version: String::new(),
            encoding: None,
            standalone: None,
        }
    }
}
