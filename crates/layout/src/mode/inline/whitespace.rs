use css_style::ComputedStyle;
use css_values::text::Whitespace;

use crate::mode::inline::collection::InlineItem;

/// Canonicalise whitespace in the collected inline items according to the CSS
/// `white-space` property of each text run, collapsing runs of whitespace into a single
/// space where appropriate and stripping leading/trailing whitespace from lines.
pub fn canonicalize_whitespace(items: &mut Vec<InlineItem>) {
    let mut last_was_space = false;
    let mut write_idx = 0;

    for read_idx in 0..items.len() {
        let item = std::mem::replace(
            &mut items[read_idx],
            InlineItem::Break {
                line_height_px: 0.0,
            },
        );

        match item {
            InlineItem::TextRun(mut text) => {
                let whitespace_prop = &text.style.whitespace;

                if matches!(whitespace_prop, Whitespace::Pre | Whitespace::PreWrap) {
                    items[write_idx] = InlineItem::TextRun(text);
                    write_idx += 1;
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
                        text.content = new_text;
                        items[write_idx] = InlineItem::TextRun(text);
                        write_idx += 1;
                    }
                }
            }
            InlineItem::InlineBoxStart { .. } | InlineItem::InlineBoxEnd { .. } => {
                items[write_idx] = item;
                write_idx += 1;
            }
            other => {
                items[write_idx] = other;
                write_idx += 1;
                last_was_space = false;
            }
        }
    }

    items.truncate(write_idx);
    strip_edge_whitespace(items);
}

/// Returns true if the given style's `white-space` property preserves
/// spaces (i.e. is `pre` or `pre-wrap`).
const fn preserves_spaces(style: &ComputedStyle) -> bool {
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
                if preserves_spaces(text.style) {
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
                if preserves_spaces(text.style) {
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

#[cfg(test)]
mod tests {
    use css_style::{ComputedStyle, StyledNode};
    use html_dom::NodeId;

    use super::*;

    #[test]
    fn collapses_whitespace_across_inline_box_boundaries() {
        let style = ComputedStyle::default();
        let mut items = vec![
            InlineItem::TextRun(crate::mode::inline::collection::TextRun {
                id: NodeId(1),
                content: "A ".to_string(),
                style: &style,
            }),
            InlineItem::InlineBoxStart {
                id: NodeId(2),
                style: &style,
            },
            InlineItem::TextRun(crate::mode::inline::collection::TextRun {
                id: NodeId(3),
                content: " ".to_string(),
                style: &style,
            }),
            InlineItem::InlineBoxEnd { id: NodeId(2) },
            InlineItem::TextRun(crate::mode::inline::collection::TextRun {
                id: NodeId(4),
                content: "B".to_string(),
                style: &style,
            }),
        ];

        canonicalize_whitespace(&mut items);

        assert_eq!(items.len(), 4);
        match &items[0] {
            InlineItem::TextRun(text) => assert_eq!(text.content, "A "),
            _ => panic!("expected leading text run"),
        }
        assert!(matches!(items[1], InlineItem::InlineBoxStart { .. }));
        assert!(matches!(items[2], InlineItem::InlineBoxEnd { .. }));
        match &items[3] {
            InlineItem::TextRun(text) => assert_eq!(text.content, "B"),
            _ => panic!("expected trailing text run"),
        }
    }

    #[test]
    fn atomic_items_still_reset_whitespace_collapse_state() {
        let style = ComputedStyle::default();
        let flow_root = StyledNode::new(NodeId(9));
        let mut items = vec![
            InlineItem::TextRun(crate::mode::inline::collection::TextRun {
                id: NodeId(1),
                content: "A ".to_string(),
                style: &style,
            }),
            InlineItem::InlineFlowRoot {
                node: &flow_root,
                style: &flow_root.style,
            },
            InlineItem::TextRun(crate::mode::inline::collection::TextRun {
                id: NodeId(2),
                content: " B".to_string(),
                style: &style,
            }),
        ];

        canonicalize_whitespace(&mut items);

        assert_eq!(items.len(), 3);
        match &items[2] {
            InlineItem::TextRun(text) => assert_eq!(text.content, " B"),
            _ => panic!("expected trailing text run"),
        }
    }
}
