use std::sync::Arc;

use io::{CacheEntry, CacheRead};
use layout::{Color4f, LayoutNode, LayoutTree, Rect};
use renderer::{DecodedImageData, ImageRenderInfo, RenderRect, RenderTri, TextBlockInfo, image::ImageCache};

use crate::{core::ScrollOffset, renderer::program::HtmlRenderer};

const IMAGE_PLACEHOLDER_COLOR: Color4f = Color4f::rgba(0.8, 0.8, 0.8, 1.0);

/// Helper function to determine if a layout node is within the visible viewport based on its dimensions and the current scroll offset.
pub fn is_visible_node(node_dimensions: Rect, initial_bounds: Rect, scroll_offset: ScrollOffset) -> bool {
    let viewport_top = f64::from(scroll_offset.y) - initial_bounds.y;
    let viewport_bottom = f64::from(scroll_offset.y) + initial_bounds.height + initial_bounds.y;
    let viewport_left = f64::from(scroll_offset.x) - initial_bounds.x;
    let viewport_right = f64::from(scroll_offset.x) + initial_bounds.width + initial_bounds.x;

    let node_bottom = node_dimensions.y + node_dimensions.height;
    let node_right = node_dimensions.x + node_dimensions.width;

    node_bottom >= viewport_top
        && node_dimensions.y <= viewport_bottom
        && node_right >= viewport_left
        && node_dimensions.x <= viewport_right
}

/// Helper function to collect all render data from a layout tree with viewport culling
pub fn collect_render_data_from_layout<'html>(
    renderer: &mut HtmlRenderer<'html>,
    layout_tree: &'html LayoutTree,
    image_cache: Option<&ImageCache>,
    initial_bounds: Rect,
    scroll_offset: ScrollOffset,
) {
    fn collect_node(
        node: &LayoutNode,
        renderer: &mut HtmlRenderer,
        image_cache: Option<&ImageCache>,
        initial_bounds: Rect,
        scroll_offset: ScrollOffset,
    ) {
        if !is_visible_node(node.dimensions, initial_bounds, scroll_offset) {
            return;
        }

        let border = node.border;
        let border_color = &node.colors.border_color;
        if (border.top > 0.0 || border.right > 0.0 || border.bottom > 0.0 || border.left > 0.0)
            && node.dimensions.width > 0.0
            && node.dimensions.height > 0.0
        {
            let x = node.dimensions.x as f32;
            let y = node.dimensions.y as f32;
            let w = node.dimensions.width as f32;
            let h = node.dimensions.height as f32;

            let inner_x = x + border.left as f32;
            let inner_y = y + border.top as f32;
            let inner_w = (w - border.horizontal() as f32).max(0.0);
            let inner_h = (h - border.vertical() as f32).max(0.0);
            let inner_right = inner_x + inner_w;
            let inner_bottom = inner_y + inner_h;

            let outer_right = x + w;
            let outer_bottom = y + h;

            if border.top > 0.0 && border_color.top.a > 0.0 {
                renderer.tris.push(RenderTri {
                    p0: [x, y],
                    p1: [outer_right, y],
                    p2: [inner_right, inner_y],
                    color: border_color.top,
                });
                renderer.tris.push(RenderTri {
                    p0: [x, y],
                    p1: [inner_right, inner_y],
                    p2: [inner_x, inner_y],
                    color: border_color.top,
                });
            }

            if border.right > 0.0 && border_color.right.a > 0.0 {
                renderer.tris.push(RenderTri {
                    p0: [outer_right, y],
                    p1: [outer_right, outer_bottom],
                    p2: [inner_right, inner_bottom],
                    color: border_color.right,
                });
                renderer.tris.push(RenderTri {
                    p0: [outer_right, y],
                    p1: [inner_right, inner_bottom],
                    p2: [inner_right, inner_y],
                    color: border_color.right,
                });
            }

            if border.bottom > 0.0 && border_color.bottom.a > 0.0 {
                renderer.tris.push(RenderTri {
                    p0: [outer_right, outer_bottom],
                    p1: [x, outer_bottom],
                    p2: [inner_x, inner_bottom],
                    color: border_color.bottom,
                });
                renderer.tris.push(RenderTri {
                    p0: [outer_right, outer_bottom],
                    p1: [inner_x, inner_bottom],
                    p2: [inner_right, inner_bottom],
                    color: border_color.bottom,
                });
            }

            if border.left > 0.0 && border_color.left.a > 0.0 {
                renderer.tris.push(RenderTri {
                    p0: [x, outer_bottom],
                    p1: [x, y],
                    p2: [inner_x, inner_y],
                    color: border_color.left,
                });
                renderer.tris.push(RenderTri {
                    p0: [x, outer_bottom],
                    p1: [inner_x, inner_y],
                    p2: [inner_x, inner_bottom],
                    color: border_color.left,
                });
            }
        }

        if node.colors.background_color.a > 0.0 {
            let border = node.border;
            let inner_x = node.dimensions.x + border.left;
            let inner_y = node.dimensions.y + border.top;
            let inner_width = (node.dimensions.width - border.horizontal()).max(0.0);
            let inner_height = (node.dimensions.height - border.vertical()).max(0.0);
            renderer.rects.push(RenderRect {
                rect: Rect::new(inner_x, inner_y, inner_width, inner_height),
                background: node.colors.background_color,
            });
        }

        if let Some(buffer) = &node.text_buffer {
            let text_block = TextBlockInfo::from_arc_buffer(
                buffer,
                node.dimensions.x as f32,
                node.dimensions.y as f32,
                node.colors.color,
            );
            if !text_block.glyphs.is_empty() {
                renderer.text_blocks.push(text_block);
            }
        }

        if let Some(image_data) = &node.image_data
            && let Some(cache) = image_cache
        {
            let src = &image_data.image_src;
            let vary_key = &image_data.vary_key;
            if let Ok(CacheEntry::Loaded(decoded)) = cache.get_with_vary(src, vary_key)
                && let CacheRead::Hit(decoded) = (*decoded).clone()
            {
                renderer.images.push(ImageRenderInfo {
                    src: src.clone(),
                    screen_rect: node.dimensions,
                    data: Arc::new(DecodedImageData {
                        rgba: decoded.rgba.clone(),
                        width: decoded.width,
                        height: decoded.height,
                    }),
                });
            } else {
                renderer.rects.push(RenderRect {
                    rect: node.dimensions,
                    background: IMAGE_PLACEHOLDER_COLOR,
                });
            }
        }

        for child in &node.children {
            collect_node(child, renderer, image_cache, initial_bounds, scroll_offset);
        }
    }

    for root in &layout_tree.root_nodes {
        collect_node(root, renderer, image_cache, initial_bounds, scroll_offset);
    }
}
