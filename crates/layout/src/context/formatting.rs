use css_display::BoxNode;
use css_style::{ComputedStyle, Display, Position};
use css_values::display::{Float, InsideDisplay, InternalDisplay, OutsideDisplay};
use html_dom::{DocumentRoot, HtmlTag};

pub(crate) struct FormattingContext;

impl FormattingContext {
    pub fn establishes_bfc(
        node: &BoxNode,
        parent_style: &ComputedStyle,
        style: &ComputedStyle,
        dom: &DocumentRoot,
    ) -> bool {
        if let Some(node_id) = node.node_id
            && let Some(element) = dom[node_id].data.as_element()
            && element.tag == HtmlTag::Html.into()
        {
            return true;
        }

        if !matches!(style.float, Float::None) {
            return true;
        }

        if matches!(style.position, Position::Absolute | Position::Fixed) {
            return true;
        }

        if matches!(
            style.display,
            Display::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::FlowRoot),
            } | Display::Normal {
                outside: None,
                inside: Some(InsideDisplay::FlowRoot),
            } | Display::Normal {
                outside: Some(OutsideDisplay::Inline),
                inside: Some(InsideDisplay::Table),
            } | Display::Internal(
                InternalDisplay::TableRow
                    | InternalDisplay::TableRowGroup
                    | InternalDisplay::TableHeaderGroup
                    | InternalDisplay::TableFooterGroup
            )
        ) {
            return true;
        }

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

        // TODO: Block elements where overflow has a value other than visible and clip.
        // TODO: Elements with contain: layout, content, or paint.
        // TODO: Query containers (elements where container-type isn't normal).
        // TODO: Multicol containers (elements where column-count or column-width isn't auto, including elements with column-count: 1).
        // TODO: column-span: all, even when the column-span: all element isn't contained by a multicol container.

        false
    }

    fn is_flex_or_grid(display: Option<InsideDisplay>) -> bool {
        matches!(display, Some(InsideDisplay::Flex | InsideDisplay::Grid))
    }

    fn is_flex_grid_or_table(display: Option<InsideDisplay>) -> bool {
        matches!(display, Some(InsideDisplay::Flex | InsideDisplay::Grid | InsideDisplay::Table))
    }
}
