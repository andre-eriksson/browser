use std::sync::Arc;

use css_style::{StyledNode, types::display::BoxDisplay};

use crate::{
    Color4f, LayoutColors, LayoutEngine, LayoutNode, Rect, SideOffset, TextContext,
    layout::LayoutContext, resolver::PropertyResolver,
};

pub struct BlockLayout;

impl BlockLayout {
    pub fn layout(
        styled_node: &StyledNode,
        ctx: &LayoutContext,
        flow_y: f32,
        text_ctx: &mut TextContext,
    ) -> LayoutNode {
        if styled_node.style.display.box_display == Some(BoxDisplay::None) {
            return LayoutNode {
                node_id: styled_node.node_id,
                dimensions: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
                colors: LayoutColors::default(),
                resolved_margin: SideOffset::zero(),
                resolved_padding: SideOffset::zero(),
                text_buffer: None,
                children: vec![],
            };
        }

        let font_size_px = styled_node.style.computed_font_size_px;

        let margin = PropertyResolver::resolve_margins(
            styled_node,
            ctx.containing_block.width,
            font_size_px,
        );
        let padding = PropertyResolver::resolve_padding(
            styled_node,
            ctx.containing_block.width,
            font_size_px,
        );

        let colors = LayoutColors {
            background_color: Color4f::from_css_color(&styled_node.style.background_color),
            color: Color4f::from_css_color(&styled_node.style.color),
        };

        let x = ctx.containing_block.x + margin.left + padding.left;
        let y = ctx.containing_block.y + flow_y + margin.top + padding.top;

        let content_width = PropertyResolver::calculate_width(
            styled_node,
            ctx.containing_block.width,
            &margin,
            &padding,
        );

        let child_ctx = LayoutContext {
            containing_block: Rect {
                x,
                y,
                width: content_width,
                height: ctx.containing_block.height,
            },
        };

        let (content_height, children, buffer) = if let Some(text) = &styled_node.text_content {
            let (_, text_height, buffer) = text_ctx.measure_text(
                text,
                font_size_px,
                &styled_node.style.line_height,
                &styled_node.style.font_family,
                content_width,
            );

            (text_height, vec![], Some(Arc::new(buffer)))
        } else {
            let mut child_flow_y = 0.0;
            let children: Vec<LayoutNode> = styled_node
                .children
                .iter()
                .map(|child| {
                    let child_node =
                        LayoutEngine::layout_node(child, &child_ctx, child_flow_y, text_ctx);
                    child_flow_y += child_node.margin_box_height();
                    child_node
                })
                .collect();

            let content_height = PropertyResolver::calculate_height(
                styled_node,
                ctx.containing_block.height,
                child_flow_y,
            );
            (content_height, children, None)
        };

        let dimensions = Rect {
            x,
            y,
            width: content_width,
            height: content_height,
        };

        LayoutNode {
            node_id: styled_node.node_id,
            dimensions,
            colors,
            resolved_margin: margin,
            resolved_padding: padding,
            text_buffer: buffer,
            children,
        }
    }
}
