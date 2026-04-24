use css_style::{ComputedSize, ComputedStyle, StyledNode};
use css_values::display::{InsideDisplay, OutsideDisplay};
use html_dom::{DocumentRoot, HtmlTag, NodeData, NodeId, Tag};

use crate::ImageContext;

#[derive(Debug, Clone)]
pub struct TextRun<'node> {
    pub id: NodeId,
    pub content: String,
    pub style: &'node ComputedStyle,
}

#[derive(Debug, Clone)]
pub struct ImageItem<'node> {
    pub id: NodeId,
    pub src: String,
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
        id: NodeId,
        style: &'node ComputedStyle,
    },

    /// Marks the closing edge of an inline element.
    /// Contributes right border + right padding and finalises the decoration.
    InlineBoxEnd { id: NodeId },

    /// inline-block or inline flow-root
    InlineFlowRoot {
        node: &'node StyledNode,
        style: &'node ComputedStyle,
    },

    /// An `<img>` element with an optional source URL and explicit dimensions.
    Image(ImageItem<'node>),

    /// A line break, <br>
    Break { line_height_px: f64 },
}

/// Recursively collects inline items from the given styled node and its children,
/// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
pub fn collect<'node>(
    dom_tree: &DocumentRoot,
    style: &'node ComputedStyle,
    inline_node: &'node StyledNode,
    items: &mut Vec<InlineItem<'node>>,
    image_ctx: &ImageContext,
) -> Result<(), ()> {
    let Some(node) = dom_tree.get_node(&inline_node.node_id) else {
        return Ok(());
    };

    match &node.data {
        NodeData::Text(content) => {
            items.push(InlineItem::TextRun(TextRun {
                id: inline_node.node_id,
                content: content.clone(),
                style,
            }));
        }
        NodeData::Element(element) => match element.tag {
            Tag::Html(HtmlTag::Br) => {
                items.push(InlineItem::Break {
                    line_height_px: inline_node.style.line_height,
                });
            }
            Tag::Html(HtmlTag::Img) => {
                const DEFAULT_IMAGE_WIDTH: f64 = 300.0;
                const DEFAULT_IMAGE_HEIGHT: f64 = 150.0;

                let Some(attrs) = element.attributes.as_ref() else {
                    return Ok(());
                };

                let src = attrs.get("src").cloned().unwrap_or_default();

                let known = image_ctx.get(&src);

                let attr_width = attrs.get("width").and_then(|v| v.parse::<f64>().ok());
                let attr_height = attrs.get("height").and_then(|v| v.parse::<f64>().ok());

                let css_width = !matches!(inline_node.style.width, ComputedSize::Auto);
                let css_height = !matches!(inline_node.style.height, ComputedSize::Auto);
                let has_explicit_width = css_width || attr_width.is_some();
                let has_explicit_height = css_height || attr_height.is_some();

                let (width, height, needs_intrinsic_size) = {
                    let w = if css_width {
                        inline_node.style.intrinsic_width
                    } else if let Some(attr_w) = attr_width {
                        attr_w
                    } else {
                        known.map_or(DEFAULT_IMAGE_WIDTH, |m| m.0)
                    };

                    let h = if css_height {
                        inline_node.style.intrinsic_height
                    } else if let Some(attr_h) = attr_height {
                        attr_h
                    } else {
                        known.map_or(DEFAULT_IMAGE_HEIGHT, |m| m.1)
                    };

                    (
                        if inline_node.style.max_intrinsic_width > 0.0 {
                            w.min(inline_node.style.max_intrinsic_width)
                        } else {
                            w
                        },
                        if inline_node.style.max_intrinsic_height > 0.0 {
                            h.min(inline_node.style.max_intrinsic_height)
                        } else {
                            h
                        },
                        attr_width.is_none() && attr_height.is_none() && !css_width && !css_height,
                    )
                };

                items.push(InlineItem::Image(ImageItem {
                    id: inline_node.node_id,
                    src,
                    width,
                    height,
                    has_explicit_width,
                    has_explicit_height,
                    needs_intrinsic_size,
                    style: &inline_node.style,
                }));
            }
            _ => {
                let display = inline_node.style.display;

                if display.outside() == Some(OutsideDisplay::Inline)
                    && display.inside() == Some(InsideDisplay::FlowRoot)
                {
                    items.push(InlineItem::InlineFlowRoot {
                        node: inline_node,
                        style: &inline_node.style,
                    });

                    return Ok(());
                } else if display.outside() != Some(OutsideDisplay::Inline) {
                    return Err(());
                }

                items.push(InlineItem::InlineBoxStart {
                    id: inline_node.node_id,
                    style: &inline_node.style,
                });

                for child in &inline_node.children {
                    collect(dom_tree, &inline_node.style, child, items, image_ctx)?;
                }

                items.push(InlineItem::InlineBoxEnd {
                    id: inline_node.node_id,
                });
            }
        },
    }

    Ok(())
}
