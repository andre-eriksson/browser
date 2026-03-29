use css_style::ComputedStyle;
use css_values::text::Whitespace;

use crate::mode::inline::collection::{InlineItem, TextRun};

/// Canonicalise whitespace in the collected inline items according to the CSS
/// `white-space` property of each text run, collapsing runs of whitespace into a single
/// space where appropriate and stripping leading/trailing whitespace from lines.
pub(crate) fn canonicalize_whitespace(items: Vec<InlineItem>) -> Vec<InlineItem> {
    let mut result = Vec::new();
    let mut last_was_space = false;

    for item in items {
        match item {
            InlineItem::TextRun(text) => {
                let whitespace_prop = &text.style.whitespace;

                if matches!(whitespace_prop, Whitespace::Pre | Whitespace::PreWrap) {
                    result.push(InlineItem::TextRun(text));
                    last_was_space = false;
                } else {
                    let mut new_text = String::with_capacity(text.content.len());

                    for c in text.content.chars() {
                        if c.is_whitespace() {
                            if matches!(whitespace_prop, Whitespace::PreLine) && c == '\n' {
                                new_text.push('\n');
                                last_was_space = false;
                            } else if !last_was_space {
                                new_text.push(' ');
                                last_was_space = true;
                            }
                        } else {
                            new_text.push(c);
                            last_was_space = false;
                        }
                    }

                    if !new_text.is_empty() {
                        result.push(InlineItem::TextRun(TextRun {
                            id: text.id,
                            content: new_text,
                            style: text.style,
                        }));
                    }
                }
            }
            other => {
                result.push(other);
                last_was_space = false;
            }
        }
    }

    strip_edge_whitespace(&mut result);

    result
}

/// Returns true if the given style's `white-space` property preserves
/// spaces (i.e. is `pre` or `pre-wrap`).
fn preserves_spaces(style: &ComputedStyle) -> bool {
    matches!(style.whitespace, Whitespace::Pre | Whitespace::PreWrap)
}

/// Strips leading and trailing whitespace from the line, removing text runs that are entirely
/// whitespace and trimming text runs at the edges. Stops stripping once it encounters a
/// text run with a style that preserves spaces.
fn strip_edge_whitespace(items: &mut Vec<InlineItem>) {
    let mut start_idx = 0;
    let mut end_idx = items.len();

    while start_idx < end_idx {
        match &items[start_idx] {
            InlineItem::TextRun(text) => {
                if preserves_spaces(&text.style) {
                    break;
                }
                let trimmed = text.content.trim_start();
                if trimmed.is_empty() {
                    start_idx += 1;
                } else {
                    let t = trimmed.to_string();
                    if let InlineItem::TextRun(text) = &mut items[start_idx] {
                        text.content = t;
                    }
                    break;
                }
            }
            _ => {
                break;
            }
        }
    }

    while end_idx > start_idx {
        match &items[end_idx - 1] {
            InlineItem::TextRun(text) => {
                if preserves_spaces(&text.style) {
                    break;
                }
                let trimmed = text.content.trim_end();
                if trimmed.is_empty() {
                    end_idx -= 1;
                } else {
                    let t = trimmed.to_string();
                    if let InlineItem::TextRun(text) = &mut items[end_idx - 1] {
                        text.content = t;
                    }
                    break;
                }
            }
            _ => {
                break;
            }
        }
    }

    if start_idx > 0 {
        items.drain(0..start_idx);
        end_idx -= start_idx;
    }

    if end_idx < items.len() {
        items.drain(end_idx..);
    }
}
