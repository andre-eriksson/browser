use css_display::BoxNode;
use css_style::{ComputedStyle, Display, Position};
use css_values::{
    OverflowBlock,
    display::{Float, InsideDisplay, InternalDisplay, OutsideDisplay},
};
use html_dom::{DocumentRoot, HtmlTag};

pub(crate) struct FormattingContext;

impl FormattingContext {
    /// Returns whether the given node establishes a block formatting context.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/CSS/Guides/Display/Block_formatting_context>
    pub fn establishes_bfc(
        node: &BoxNode,
        parent_style: &ComputedStyle,
        style: &ComputedStyle,
        dom: &DocumentRoot,
    ) -> bool {
        // * The root element of the document (<html>).
        if let Some(node_id) = node.node_id
            && let Some(element) = dom[node_id].data.as_element()
            && element.tag == HtmlTag::Html.into()
        {
            return true;
        }

        // * Floats (elements where float isn't none).
        if !matches!(style.float, Float::None) {
            return true;
        }

        // * Absolutely positioned elements (elements where position is absolute or fixed).
        if matches!(style.position, Position::Absolute | Position::Fixed) {
            return true;
        }

        // * Elements with display: flow-root.
        // * Anonymous table cells implicitly created by the elements with
        //   display: table, table-row, table-row-group, table-header-group,
        //   table-footer-group (which is the default for HTML tables, table
        //   rows, table bodies, table headers, and table footers, respectively),
        //   or inline-table.
        if matches!(
            style.display,
            Display::Normal {
                inside: InsideDisplay::FlowRoot,
                ..
            } | Display::Normal {
                outside: OutsideDisplay::Inline,
                inside: InsideDisplay::Table,
            } | Display::Internal(
                InternalDisplay::TableRow
                    | InternalDisplay::TableRowGroup
                    | InternalDisplay::TableHeaderGroup
                    | InternalDisplay::TableFooterGroup
            )
        ) {
            return true;
        }

        // * Flex items (direct children of the element with display: flex or inline-flex)
        //   if they are neither flex nor grid nor table containers themselves.
        // * Grid items (direct children of the element with display: grid or inline-grid)
        //   if they are neither flex nor grid nor table containers themselves.
        if let (
            Display::Normal {
                inside: parent_inside,
                ..
            },
            Display::Normal {
                inside: current_inside,
                ..
            },
        ) = (parent_style.display, style.display)
        {
            let parent_is_flex_or_grid = Self::is_flex_or_grid(parent_inside);
            let current_is_excluded = Self::is_flex_grid_or_table(current_inside);

            if parent_is_flex_or_grid && !current_is_excluded {
                return true;
            }
        }

        // * Block elements where overflow has a value other than visible and clip.
        if style.display.is_block()
            && (!matches!(style.overflow_x, OverflowBlock::Visible | OverflowBlock::Clip)
                || !matches!(style.overflow_y, OverflowBlock::Visible | OverflowBlock::Clip))
        {
            return true;
        }

        // TODO: Elements with contain: layout, content, or paint.
        // TODO: Query containers (elements where container-type isn't normal).
        // TODO: Multicol containers (elements where column-count or column-width isn't auto, including elements with column-count: 1).
        // TODO: column-span: all, even when the column-span: all element isn't contained by a multicol container.

        false
    }

    fn is_flex_or_grid(display: InsideDisplay) -> bool {
        matches!(display, InsideDisplay::Flex | InsideDisplay::Grid)
    }

    fn is_flex_grid_or_table(display: InsideDisplay) -> bool {
        matches!(display, InsideDisplay::Flex | InsideDisplay::Grid | InsideDisplay::Table)
    }
}
