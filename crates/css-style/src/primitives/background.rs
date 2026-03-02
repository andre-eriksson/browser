#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum Attachment {
    /// The background scrolls along with the element's content. This is the default value for the `background-attachment` property.
    #[default]
    Scroll,

    /// The background is fixed with regard to the viewport. It does not move when the content of the element is scrolled.
    Fixed,

    /// The background scrolls along with the element's content, but only within the bounds of the element itself. When the content of the element is scrolled, the background will move, but it will not scroll outside of the element's area.
    Local,
}

/// The `background-origin` property specifies the background painting area for an element. It determines where the
/// background image or color is applied in relation to the content, padding, and border of the element.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
pub enum VisualBox {
    /// The background is painted within the content box, which is the area where the content of the element is displayed.
    Content,

    /// The background is painted within the padding box, which includes the content box and the padding area around it.
    /// The background will extend into the padding area but not into the border area.
    /// This is the default value for the `background-origin` property, meaning that if no value is specified,
    /// the background will be painted within the padding box.
    Padding,

    /// The background is painted within the border box, which includes the content box, padding box, and border area. T
    /// he background will extend into both the padding and border areas.
    #[default]
    Border,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Clip {
    Text,
    BorderArea,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgClip {
    Visual(VisualBox),
    Clip(Clip, Option<Clip>),
}

impl Default for BgClip {
    fn default() -> Self {
        BgClip::Visual(VisualBox::default())
    }
}
