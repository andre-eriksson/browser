use css_style::{ComputedSize, ComputedMaxDimension, ComputedStyle};

use crate::{
    ImageData, LayoutColors, LayoutNode, Rect, TextContext,
    layout::LayoutContext,
    mode::inline::{InlineLayoutContext, collection::ImageItem, line::LineBoxBuilder},
};

pub fn layout_image<'node>(
    ctx: &mut InlineLayoutContext<'node>,
    img: &ImageItem,
    text_ctx: &mut TextContext,
    layout_ctx: &LayoutContext,
    line: &mut LineBoxBuilder<'node>,
) {
    let alignment = &img.style.text_align;
    let writing_mode = &img.style.writing_mode;
    text_ctx.last_text_align = *alignment;
    text_ctx.last_writing_mode = *writing_mode;
    let has_intrinsic_size = layout_ctx
        .image_ctx()
        .get(&img.src)
        .is_some_and(|(w, h)| w > 0.0 && h > 0.0);

    let (img_width, img_height) = resolve_image_size(
        img.width,
        img.height,
        img.has_explicit_width,
        img.has_explicit_height,
        img.style,
        ctx.available_width,
        layout_ctx.image_ctx().get(&img.src),
    );

    if line.line_box.width + img_width > ctx.available_width && line.line_box.width > 0.0 {
        line.finish_line_with_decorations(ctx, text_ctx, layout_ctx.float_ctx_ref(), None);
    }

    let node = LayoutNode::builder(img.id)
        .dimensions(Rect::new(0.0, 0.0, img_width, img_height))
        .colors(LayoutColors::from(img.style))
        .image_data(ImageData {
            image_src: img.src.clone(),
            vary_key: layout_ctx
                .image_ctx()
                .get_meta(&img.src)
                .map(|m| m.vary_key.clone())
                .unwrap_or_default(),
            image_needs_intrinsic_size: img.needs_intrinsic_size && !has_intrinsic_size,
        })
        .build();

    let ascent = img_height;
    line.line_box.add(node, ascent, 0.0);
}

fn resolve_image_size(
    width: f64,
    height: f64,
    has_explicit_width: bool,
    has_explicit_height: bool,
    style: &ComputedStyle,
    available_width: f64,
    intrinsic_size: Option<(f64, f64)>,
) -> (f64, f64) {
    let max_width = match style.max_width {
        ComputedMaxDimension::Percentage(f) => (available_width * f).max(0.0),
        _ => style.max_intrinsic_width,
    };
    let mut used_width = match style.width {
        ComputedSize::Percentage(f) => (available_width * f).max(0.0),
        _ => width.max(0.0),
    };
    let mut used_height = height.max(0.0);

    if let Some((intrinsic_width, intrinsic_height)) = intrinsic_size.filter(|(w, h)| *w > 0.0 && *h > 0.0) {
        if has_explicit_width && !has_explicit_height {
            used_height = used_width * intrinsic_height / intrinsic_width;
        } else if has_explicit_height && !has_explicit_width {
            used_width = used_height * intrinsic_width / intrinsic_height;
        } else if !has_explicit_width && !has_explicit_height {
            used_width = intrinsic_width;
            used_height = intrinsic_height;
        }

        if has_explicit_width && has_explicit_height {
            used_width = used_width.min(max_width);
            used_height = used_height.min(style.max_intrinsic_height);
        } else {
            let width_scale = if used_width > max_width {
                max_width / used_width
            } else {
                1.0
            };
            let height_scale = if used_height > style.max_intrinsic_height {
                style.max_intrinsic_height / used_height
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
        used_height = used_height.min(style.max_intrinsic_height);
    }

    (used_width.max(0.0), used_height.max(0.0))
}
