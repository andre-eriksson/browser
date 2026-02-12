//! The `display` property specifies the display behavior (the type of rendering box) of an element in the document tree.
//! It is a shorthand property for setting:
//! * `display-outside`
//! * `display-inside`
//! * `display-listitem`
//! * `display-internal`
//! * `display-box`
//!
//! <https://developer.mozilla.org/en-US/docs/Web/CSS/Reference/Properties/display>

use strum::EnumString;

/// These keywords specify the element's outer display type, which is essentially its role in flow layout:
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum OutsideDisplay {
    /// The element generates a block box, generating line breaks both before and after the element when in the normal flow.
    Block,

    /// The element generates one or more inline boxes that do not generate line breaks before or after themselves.
    /// In normal flow, the next element will be on the same line if there is space.
    Inline,
}

/// These keywords specify the element's inner display type, which defines the type of formatting context that its
/// contents are laid out in (assuming it is a non-replaced element). When one of these keywords is used by itself
/// as a single value, the element's outer display type defaults to block (with the exception of ruby,
/// which defaults to inline).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum InsideDisplay {
    /// The element lays out its contents using flow layout (block-and-inline layout). If its outer display type is inline, and it is participating
    /// in a block or inline formatting context, then it generates an inline box. Otherwise it generates a block box. Depending on the value of
    /// other properties (such as position, float, or overflow) and whether it is itself participating in a block or inline formatting context,
    /// it either establishes a new block formatting context (BFC) for its contents or integrates its contents into its parent formatting context.
    Flow,

    /// The element generates a block box that establishes a new block formatting context, defining where the formatting root lies.
    FlowRoot,

    /// These elements behave like HTML `<table>` elements. It defines a block-level box.
    Table,

    /// The element behaves like a block-level element and lays out its content according to the flexbox model.
    Flex,

    /// The element behaves like a block-level element and lays out its content according to the grid model.
    Grid,

    /// The element behaves like an inline-level element and lays out its content according to the ruby formatting model.
    /// It behaves like the corresponding HTML `<ruby>` elements.
    Ruby,
}

/// The element generates a block box for the content and a separate list-item inline box.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum ListItemDisplay {
    /// A single value of list-item will cause the element to behave like a list item. This can be used together with list-style-type and list-style-position.
    /// list-item can also be combined with any `<display-outside>` keyword and the flow or flow-root `<display-inside>` keywords.
    ListItem,
}

/// Some layout models such as table and ruby have a complex internal structure, with several different roles that their children and descendants can fill.
/// This section defines those "internal" display values, which only have meaning within that particular layout mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum InternalDisplay {
    /// These elements behave like `<tbody>` HTML elements.
    TableRowGroup,

    /// These elements behave like `<thead>` HTML elements.
    TableHeaderGroup,

    /// These elements behave like `<tfoot>` HTML elements.
    TableFooterGroup,

    /// These elements behave like `<tr>` HTML elements.
    TableRow,

    /// These elements behave like `<td>` HTML elements.
    TableCell,

    /// These elements behave like `<colgroup>` HTML elements.
    TableColumnGroup,

    /// These elements behave like `<col>` HTML elements.
    TableColumn,

    /// These elements behave like `<caption>` HTML elements.
    TableCaption,

    /// These elements behave like `<rb>` HTML elements.
    RubyBase,

    /// These elements behave like `<rt>` HTML elements.
    RubyText,

    /// These elements are generated as anonymous boxes.
    RubyBaseContainer,

    /// These elements behave like `<rtc>` HTML elements.
    RubyTextContainer,
}

/// These values define whether an element generates display boxes at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, EnumString)]
#[strum(serialize_all = "kebab_case", ascii_case_insensitive, parse_err_ty = String, parse_err_fn = String::from)]
pub enum BoxDisplay {
    /// These elements don't produce a specific box by themselves. They are replaced by their pseudo-box and their child boxes.
    /// Please note that the CSS Display Level 3 spec defines how the contents value should affect "unusual elements" — elements
    /// that aren't rendered purely by CSS box concepts such as replaced elements.
    Contents,

    /// Turns off the display of an element so that it has no effect on layout (the document is rendered as though the element did not exist).
    /// All descendant elements also have their display turned off. To have an element take up the space that it would normally take, but without
    /// actually rendering anything, use the visibility property instead.
    None,
}
