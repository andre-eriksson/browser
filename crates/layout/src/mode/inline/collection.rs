use std::sync::Arc;

use css_style::{ComputedDimension, ComputedStyle, StyledNode};
use css_values::display::{InsideDisplay, OutsideDisplay};
use html_dom::{HtmlTag, NodeId, Tag};

use crate::ImageContext;

#[derive(Debug, Clone)]
pub struct TextRun {
    pub id: NodeId,
    pub content: String,
    pub style: Arc<ComputedStyle>,
}

#[derive(Debug, Clone)]
pub struct ImageItem {
    pub id: NodeId,
    pub src: String,
    pub width: f32,
    pub height: f32,
    pub has_explicit_width: bool,
    pub has_explicit_height: bool,
    pub needs_intrinsic_size: bool,
    pub style: Arc<ComputedStyle>,
}

/// An item in the intermediate representation of an inline layout, representing
/// either a run of text with a single style or the start/end of an inline box.
#[derive(Debug, Clone)]
pub enum InlineItem {
    /// A run of text with the same style
    TextRun(TextRun),

    /// Marks the opening edge of an inline element (e.g. `<span>`).
    /// Contributes left border + left padding to the line and begins tracking
    /// a decoration region.
    InlineBoxStart {
        id: NodeId,
        style: Arc<ComputedStyle>,
    },

    /// Marks the closing edge of an inline element.
    /// Contributes right border + right padding and finalises the decoration.
    InlineBoxEnd { id: NodeId },

    /// inline-block or inline flow-root
    InlineFlowRoot {
        node: Box<StyledNode>,
        style: Arc<ComputedStyle>,
    },

    /// An `<img>` element with an optional source URL and explicit dimensions.
    Image(ImageItem),

    /// A line break, <br>
    Break { line_height_px: f32 },
}

/// Recursively collects inline items from the given styled node and its children,
/// returning an error if it encounters a block-level element (which should be handled by the block layout instead).
pub(crate) fn collect(
    style: &ComputedStyle,
    inline_node: &StyledNode,
    items: &mut Vec<InlineItem>,
    image_ctx: &ImageContext,
) -> Result<(), ()> {
    if let Some(text) = inline_node.text_content.as_ref() {
        items.push(InlineItem::TextRun(TextRun {
            id: inline_node.node_id,
            content: text.clone(),
            style: Arc::new(style.inherited_subset()),
        }));
    }

    if let Some(tag) = inline_node.tag.as_ref() {
        match tag {
            Tag::Html(HtmlTag::Br) => {
                items.push(InlineItem::Break {
                    line_height_px: inline_node.style.line_height,
                });
            }
            Tag::Html(HtmlTag::Img) => {
                let src = inline_node
                    .attributes
                    .get("src")
                    .cloned()
                    .unwrap_or_default();

                const DEFAULT_IMAGE_WIDTH: f32 = 300.0;
                const DEFAULT_IMAGE_HEIGHT: f32 = 150.0;

                let known = image_ctx.get(&src);

                let attr_width = inline_node
                    .attributes
                    .get("width")
                    .and_then(|v| v.parse::<f32>().ok());

                let attr_height = inline_node
                    .attributes
                    .get("height")
                    .and_then(|v| v.parse::<f32>().ok());

                let css_width = !matches!(inline_node.style.width, ComputedDimension::Auto);
                let css_height = !matches!(inline_node.style.height, ComputedDimension::Auto);
                let has_explicit_width = css_width || attr_width.is_some();
                let has_explicit_height = css_height || attr_height.is_some();

                let (width, height, needs_intrinsic_size) = {
                    let w = if css_width {
                        inline_node.style.intrinsic_width
                    } else if let Some(attr_w) = attr_width {
                        attr_w
                    } else {
                        known.map(|m| m.0).unwrap_or(DEFAULT_IMAGE_WIDTH)
                    };

                    let h = if css_height {
                        inline_node.style.intrinsic_height
                    } else if let Some(attr_h) = attr_height {
                        attr_h
                    } else {
                        known.map(|m| m.1).unwrap_or(DEFAULT_IMAGE_HEIGHT)
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
                    style: Arc::new(inline_node.style.clone()),
                }));
            }
            _ => {
                let display = inline_node.style.display;

                if display.outside() == Some(OutsideDisplay::Inline)
                    && display.inside() == Some(InsideDisplay::FlowRoot)
                {
                    items.push(InlineItem::InlineFlowRoot {
                        node: inline_node.clone().into(),
                        style: Arc::new(inline_node.style.clone()),
                    });

                    return Ok(());
                } else if display.outside() != Some(OutsideDisplay::Inline) {
                    return Err(());
                }

                items.push(InlineItem::InlineBoxStart {
                    id: inline_node.node_id,
                    style: Arc::new(inline_node.style.clone()),
                });

                for child in &inline_node.children {
                    collect(&inline_node.style, child, items, image_ctx)?;
                }

                items.push(InlineItem::InlineBoxEnd {
                    id: inline_node.node_id,
                });
            }
        }
    }

    Ok(())
}
