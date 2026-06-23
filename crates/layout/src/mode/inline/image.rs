use css_style::{ComputedMaxSize, ComputedSize, ComputedStyle};

use crate::{
    ImageData, LayoutColors, LayoutInput, LayoutNode, Rect,
    context::FloatContext,
    mode::inline::{InlineLayoutContext, collection::ImageItem, line::LineBoxBuilder},
};

pub fn layout_image<'node>(
    nodes: &mut [Option<LayoutNode>],
    ctx: &mut InlineLayoutContext<'node>,
    input: &mut LayoutInput<'_>,
    img: &ImageItem,
    line: &mut LineBoxBuilder<'node>,
    float_ctx: &mut FloatContext,
) {
    let alignment = &img.style.text_align;
    let writing_mode = &img.style.writing_mode;
    input.text.last_text_align = *alignment;
    input.text.last_writing_mode = *writing_mode;

    let image = input.image.get(img.node_id);

    let has_intrinsic_size = image.as_ref().is_some_and(|i| i.width > 0 && i.height > 0);

    let (img_width, img_height) = resolve_image_size(
        img.width,
        img.height,
        img.has_explicit_width,
        img.has_explicit_height,
        img.style,
        ctx.available_width,
        image.map(|i| (i.width, i.height)),
    );

    if line.line_box.width + img_width > ctx.available_width && line.line_box.width > 0.0 {
        line.finish_line_with_decorations(nodes, ctx, input.text, float_ctx, None);
    }

    let node = LayoutNode::builder(*img.layout_id)
        .dimensions(Rect::new(0.0, 0.0, img_width, img_height))
        .colors(LayoutColors::from(img.style))
        .node_id(*img.node_id)
        .image_data(ImageData {
            node_id: *img.node_id,
            image_needs_intrinsic_size: img.needs_intrinsic_size && !has_intrinsic_size,
        })
        .build();

    let ascent = img_height;
    line.line_box.add_ascent(ascent);
    nodes[img.layout_id.index()] = Some(node);
    ctx.ids.push(*img.layout_id);
}

fn resolve_image_size(
    width: f64,
    height: f64,
    has_explicit_width: bool,
    has_explicit_height: bool,
    style: &ComputedStyle,
    available_width: f64,
    intrinsic_size: Option<(u32, u32)>,
) -> (f64, f64) {
    let max_width = match style.max_width {
        ComputedMaxSize::Px(px) => px,
        ComputedMaxSize::Percentage(f) => (available_width * f).max(0.0),
        _ => available_width.max(0.0),
    };
    let max_height = match style.max_height {
        ComputedMaxSize::Px(px) => px,
        ComputedMaxSize::Percentage(f) => (available_width * f).max(0.0),
        _ => f64::INFINITY,
    };

    let mut used_width = match style.width {
        ComputedSize::Px(px) => px,
        ComputedSize::Percentage(f) => (available_width * f).max(0.0),
        _ => width.max(0.0),
    };
    let mut used_height = height.max(0.0);

    if let Some((intrinsic_width, intrinsic_height)) = intrinsic_size.filter(|(w, h)| *w > 0 && *h > 0) {
        if has_explicit_width && !has_explicit_height {
            used_height = used_width * intrinsic_height as f64 / intrinsic_width as f64;
        } else if has_explicit_height && !has_explicit_width {
            used_width = used_height * intrinsic_width as f64 / intrinsic_height as f64;
        } else if !has_explicit_width && !has_explicit_height {
            used_width = intrinsic_width as f64;
            used_height = intrinsic_height as f64;
        }

        if has_explicit_width && has_explicit_height {
            used_width = used_width.min(max_width);
            used_height = used_height.min(max_height);
        } else {
            let width_scale = if used_width > max_width {
                max_width / used_width
            } else {
                1.0
            };
            let height_scale = if used_height > max_height {
                max_height / used_height
            } else {
                1.0
            };
            let scale = width_scale.min(height_scale);

            if scale < 1.0 {
                used_width *= scale;
                used_height *= scale;
            }
        }
    } else {
        used_width = used_width.min(max_width);
        used_height = used_height.min(max_height);
    }

    (used_width.max(0.0), used_height.max(0.0))
}
