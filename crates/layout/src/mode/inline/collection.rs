use css_display::LayoutNodeId;
use css_style::{ComputedMaxSize, ComputedSize, ComputedStyle, Display};
use css_values::display::{InsideDisplay, OutsideDisplay};
use html_dom::{HtmlTag, NodeData, NodeId, Tag};

use crate::{LayoutInput, Rect};

#[derive(Debug, Clone)]
pub struct TextRun<'node> {
    pub layout_id: &'node LayoutNodeId,
    pub node_id: &'node NodeId,
    pub content: String,
    pub style: &'node ComputedStyle,
}

#[derive(Debug, Clone)]
pub struct ImageItem<'node> {
    pub layout_id: &'node LayoutNodeId,
    pub node_id: &'node NodeId,
    pub width: f64,
    pub height: f64,
    pub has_explicit_width: bool,
    pub has_explicit_height: bool,
    pub needs_intrinsic_size: bool,
    pub style: &'node ComputedStyle,
}

/// An item in the intermediate representation of an inline layout, representing
/// either a run of text with a single style or the start/end of an inline box.
#[derive(Debug, Clone)]
pub enum InlineItem<'node> {
    /// A run of text with the same style
    TextRun(TextRun<'node>),

    /// Marks the opening edge of an inline element (e.g. `<span>`).
    /// Contributes left border + left padding to the line and begins tracking
    /// a decoration region.
    InlineBoxStart {
        layout_id: &'node LayoutNodeId,
        node_id: &'node NodeId,
        style: &'node ComputedStyle,
    },

    /// Marks the closing edge of an inline element.
    /// Contributes right border + right padding and finalises the decoration.
    InlineBoxEnd { layout_id: &'node LayoutNodeId },

    /// inline-block or inline flow-root
    InlineFlowRoot {
        layout_id: &'node LayoutNodeId,
        style: &'node ComputedStyle,
    },

    /// An `<img>` element with an optional source URL and explicit dimensions.
    Image(ImageItem<'node>),

    /// A line break, <br>
    Break { line_height_px: f64 },
}

/// Recursively collects inline items from the given styled node and its children,
/// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
pub fn collect<'dom>(
    containing_rect: Rect,
    input: &mut LayoutInput<'dom>,
    parent_style: &'dom ComputedStyle,
    layout_id: &'dom LayoutNodeId,
    items: &mut Vec<InlineItem<'dom>>,
) -> Result<(), ()> {
    let box_node = &input.box_tree[layout_id];
    let Some(node_id) = &box_node.node_id else {
        return Ok(());
    };

    let node = &input.dom[node_id];
    let style = &*box_node.style;

    match &node.data {
        NodeData::Text(content) => {
            items.push(InlineItem::TextRun(TextRun {
                layout_id,
                node_id,
                content: content.clone(),
                style: parent_style,
            }));
        }
        NodeData::Element(element) => match element.tag {
            Tag::Html(HtmlTag::Br) => {
                items.push(InlineItem::Break {
                    line_height_px: style.line_height,
                });
            }
            Tag::Html(HtmlTag::Img) => {
                const DEFAULT_IMAGE_WIDTH: f64 = 300.0;
                const DEFAULT_IMAGE_HEIGHT: f64 = 150.0;

                let Some(attrs) = element.attributes.as_ref() else {
                    return Ok(());
                };

                let known = input.image.get(&box_node.node_id.unwrap());

                let attr_width = attrs.get("width").and_then(|v| v.parse::<f64>().ok());
                let attr_height = attrs.get("height").and_then(|v| v.parse::<f64>().ok());

                let css_width = !matches!(style.width, ComputedSize::Auto);
                let css_height = !matches!(style.height, ComputedSize::Auto);
                let has_explicit_width = css_width || attr_width.is_some();
                let has_explicit_height = css_height || attr_height.is_some();

                let (width, height, needs_intrinsic_size) = {
                    let w = if css_width {
                        match style.width {
                            ComputedSize::Px(px) => px,
                            ComputedSize::Percentage(frac) => frac * containing_rect.width,
                            _ => known
                                .as_ref()
                                .map_or(DEFAULT_IMAGE_WIDTH, |m| m.width as f64), // TODO: Handle other types of computed size
                        }
                    } else if let Some(attr_w) = attr_width {
                        attr_w
                    } else {
                        known
                            .as_ref()
                            .map_or(DEFAULT_IMAGE_WIDTH, |m| m.width as f64)
                    };

                    let h = if css_height {
                        match style.height {
                            ComputedSize::Px(px) => px,
                            ComputedSize::Percentage(frac) => frac * containing_rect.height,
                            _ => known
                                .as_ref()
                                .map_or(DEFAULT_IMAGE_HEIGHT, |m| m.height as f64), // TODO: Handle other types of computed size
                        }
                    } else if let Some(attr_h) = attr_height {
                        attr_h
                    } else {
                        known
                            .as_ref()
                            .map_or(DEFAULT_IMAGE_HEIGHT, |m| m.height as f64)
                    };

                    let max_width = match style.max_width {
                        ComputedMaxSize::Px(px) => px,
                        ComputedMaxSize::Percentage(f) => f * containing_rect.width,
                        _ => f64::INFINITY,
                    };

                    let max_height = match style.max_height {
                        ComputedMaxSize::Px(px) => px,
                        _ => f64::INFINITY,
                    };

                    (
                        if max_width > 0.0 { w.min(max_width) } else { w },
                        if max_height > 0.0 {
                            h.min(max_height)
                        } else {
                            h
                        },
                        attr_width.is_none() && attr_height.is_none() && !css_width && !css_height,
                    )
                };

                items.push(InlineItem::Image(ImageItem {
                    layout_id,
                    node_id,
                    width,
                    height,
                    has_explicit_width,
                    has_explicit_height,
                    needs_intrinsic_size,
                    style,
                }));
            }
            _ => {
                let display = style.display;

                if let Display::Normal { outside, inside } = display {
                    if outside == Some(OutsideDisplay::Inline) && inside == Some(InsideDisplay::FlowRoot) {
                        items.push(InlineItem::InlineFlowRoot { layout_id, style });

                        return Ok(());
                    } else if outside != Some(OutsideDisplay::Inline) {
                        return Err(());
                    }
                }

                items.push(InlineItem::InlineBoxStart {
                    layout_id,
                    node_id,
                    style,
                });

                for child_node in &box_node.children {
                    collect(containing_rect, input, style, child_node, items)?;
                }

                items.push(InlineItem::InlineBoxEnd { layout_id });
            }
        },
    }

    Ok(())
}
