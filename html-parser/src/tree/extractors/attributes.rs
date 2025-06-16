use std::collections::HashMap;

/// A utility function to extract attributes from an HTML tag slice.
///
/// # Arguments
/// * `tag_slice` - A string slice representing the attributes of an HTML tag.
///
/// # Returns
/// A `HashMap<String, String>` where keys are attribute names and values are attribute values.
pub fn extract_attributes(tag_slice: &str) -> HashMap<String, String> {
    let mut attributes = HashMap::new();
    let chars: Vec<char> = tag_slice.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Skip whitespace
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }

        if i >= len {
            break; // End of string
        }

        // Read attribute name
        let name_start = i;
        while i < len
            && (chars[i].is_ascii_alphanumeric()
                || chars[i] == '-'
                || chars[i] == '_'
                || chars[i] == ':'
                || chars[i] == '.')
        {
            i += 1;
        }

        if i == name_start {
            i += 1;
            continue;
        }

        let name = chars[name_start..i].iter().collect::<String>();

        while i < len && chars[i].is_whitespace() {
            i += 1; // Skip whitespace after attribute name
        }

        if i < len && chars[i] == '=' {
            i += 1;

            while i < len && chars[i].is_whitespace() {
                i += 1; // Skip whitespace after '='
            }

            if i >= len {
                // Treat as boolean attribute if no value is provided
                attributes.insert(name, String::new());
                break;
            }

            let value = if chars[i] == '"' {
                i += 1;

                let value_start = i;
                while i < len && chars[i] != '"' {
                    i += 1;
                }
                let value = chars[value_start..i].iter().collect::<String>();
                if i < len {
                    i += 1;
                }
                value
            } else if chars[i] == '\'' {
                i += 1;

                let value_start = i;
                while i < len && chars[i] != '\'' {
                    i += 1;
                }
                let value = chars[value_start..i].iter().collect::<String>();
                if i < len {
                    i += 1;
                }
                value
            } else {
                let value_start = i;
                while i < len && !chars[i].is_whitespace() && chars[i] != '>' && chars[i] != '=' {
                    i += 1;
                }
                chars[value_start..i].iter().collect::<String>()
            };

            attributes.insert(name, value);
        } else {
            // If no '=' is found, treat as a boolean attribute
            attributes.insert(name, String::new());
        }
    }

    attributes
}
