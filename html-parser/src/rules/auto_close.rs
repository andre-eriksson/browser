use shared_types::dom::{DomNode, Element};

fn should_auto_close(current_tag: &str, new_tag: &str) -> bool {
    let current_lower = current_tag.to_lowercase();
    let new_lower = new_tag.to_lowercase();

    match current_lower.as_str() {
        "p" => {
            // Automatically close <p> when encountering block-level elements
            matches!(
                new_lower.as_str(),
                "div"
                    | "p"
                    | "h1"
                    | "h2"
                    | "h3"
                    | "h4"
                    | "h5"
                    | "h6"
                    | "ul"
                    | "ol"
                    | "li"
                    | "dl"
                    | "dt"
                    | "dd"
                    | "blockquote"
                    | "pre"
                    | "form"
                    | "table"
                    | "section"
                    | "article"
                    | "aside"
                    | "header"
                    | "footer"
                    | "nav"
                    | "main"
                    | "figure"
                    | "hr"
            )
        }
        "li" => {
            // Automatically close <li> when encountering another <li>
            new_lower == "li"
        }
        "dd" | "dt" => {
            // Automatically close <dd> or <dt> when encountering another <dd> or <dt>
            matches!(new_lower.as_str(), "dd" | "dt")
        }
        "option" => {
            // Automatically close <option> when encountering another <option> or <optgroup>
            matches!(new_lower.as_str(), "option" | "optgroup")
        }
        "tr" => &new_lower == "tr",
        "td" | "th" => {
            // Automatically close <td> or <th> when encountering another <td> or <th>
            matches!(new_lower.as_str(), "td" | "th" | "tr")
        }
        _ => false,
    }
}

pub fn auto_close_elements(
    element_stack: &mut Vec<Element>,
    document_children: &mut Vec<DomNode>,
    new_tag: &str,
) {
    let mut elements_to_close: Vec<usize> = Vec::new();

    for (i, element) in element_stack.iter().enumerate().rev() {
        if should_auto_close(&element.tag_name, new_tag) {
            elements_to_close.push(i);
        } else {
            // Stop closing elements if we reach one that doesn't need to be closed
            break;
        }
    }

    for &index in elements_to_close.iter().rev() {
        if let Some(element) = element_stack.get(index).cloned() {
            element_stack.remove(index);

            if element_stack.is_empty() {
                document_children.push(DomNode::Element(element));
            } else {
                if let Some(parent_element) = element_stack.last_mut() {
                    parent_element.children.push(DomNode::Element(element));
                } else {
                    document_children.push(DomNode::Element(element));
                }
            }
        }
    }
}
