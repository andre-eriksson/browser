use std::{fmt::Debug, sync::Arc};

use browser_preferences::theme::ThemeCategory;
use css_cssom::ComponentValueStream;
use css_values::{
    CSSParsable,
    border::{BorderStyle, BorderWidth},
    color::Color,
    cursor::Cursor,
    dimension::{MarginValue, MaxSize, OffsetValue, Size},
    display::{Clear, Float},
    error::CssValueError,
    flex::{FlexBasis, FlexDirection, FlexWrap},
    global::Global,
    numeric::{Flex, Order},
    text::{FontSize, FontWeight, LineHeight, TextAlign, Whitespace, WritingMode},
};
use url::Url;

use crate::{
    ComputedStyle, Display, FontFamily, Position,
    properties::background::{
        BackgroundAttachment, BackgroundBlendMode, BackgroundClip, BackgroundImage, BackgroundOrigin,
        BackgroundPositionX, BackgroundPositionY, BackgroundRepeat, BackgroundSize,
    },
};

pub mod background;
pub mod border;
pub mod color;
pub mod display;
pub mod font;
pub mod length;
pub mod offset;
pub mod percentage;
pub mod position;
pub mod text;

/// Trait for types that can be represented in pixels. This is used for properties that can be
/// specified in various units (e.g., 'em', '%', 'vw') and need to be converted to pixels for
/// layout calculations. The `to_px` method takes into account the context of the property, such
/// as the parent font size for 'em' units or the viewport dimensions for 'vw' units.
pub trait PixelRepr: Sized {
    /// Converts the value to pixels based on the provided context. The `rel_type` parameter indicates
    /// the type of relative measurement (e.g., font size, parent width) that may be needed for the conversion.
    /// The `rel_ctx` provides access to the parent style for inheritance and percentage calculations, while
    /// the `abs_ctx` provides access to absolute context values like root font size and viewport dimensions.
    fn to_px_unchecked(
        self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> f64 {
        self.to_px(rel_type, rel_ctx, abs_ctx)
            .expect("Failed to convert to pixels")
    }

    fn to_px(
        self,
        rel_type: Option<RelativeType>,
        rel_ctx: Option<&RelativeContext>,
        abs_ctx: &AbsoluteContext,
    ) -> Result<f64, String>;
}

/// Global CSS values that can be applied to any property, affecting how the property is resolved in relation to its initial value, inheritance, and user styles.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeType {
    FontSize,
    ParentWidth,
    ParentHeight,
    RootFontSize,
    ViewportWidth,
    ViewportHeight,
    BackgroundArea,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DocumentContext {
    pub root_font_size: f64,
    pub root_line_height_multiplier: f64,
    pub root_color: Color,
    pub theme_category: ThemeCategory,
}

/// Context for resolving absolute CSS properties.
#[derive(Debug, Clone, PartialEq)]
pub struct AbsoluteContext<'page> {
    pub root_font_size: f64,
    pub root_line_height_multiplier: f64,
    pub viewport_width: f64,
    pub viewport_height: f64,
    pub root_color: Color,
    pub theme_category: ThemeCategory,
    pub document_url: &'page Url,
}

impl<'page> AbsoluteContext<'page> {
    #[must_use]
    pub fn new(
        document_ctx: DocumentContext,
        viewport_width: f64,
        viewport_height: f64,
        document_url: &'page Url,
    ) -> Self {
        Self {
            root_font_size: document_ctx.root_font_size,
            root_line_height_multiplier: document_ctx.root_line_height_multiplier,
            root_color: document_ctx.root_color,
            theme_category: document_ctx.theme_category,
            viewport_width,
            viewport_height,
            document_url,
        }
    }

    #[must_use]
    pub const fn default_url(document_url: &'page Url) -> Self {
        Self {
            root_font_size: 16.0,
            root_line_height_multiplier: 1.2,
            root_color: Color::BLACK,
            theme_category: ThemeCategory::Light,
            viewport_width: 800.0,
            viewport_height: 600.0,
            document_url,
        }
    }
}

/// Context for resolving relative CSS properties, such as percentages or 'em' units. It provides access to the parent style for inheritance and percentage calculations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RelativeContext {
    pub parent: Arc<ComputedStyle>,
    pub font_size: f64,
}

/// A CSS property that can either be a specific value or a global value (initial, inherit, unset, revert, revert-layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CSSProperty<T> {
    Value(T),
    Global(Global),
}

impl<T: CSSParsable> CSSProperty<T> {
    /// Returns the specific value of the property if it is set, or None if it is a global value.
    pub(crate) const fn as_value(&self) -> Option<&T> {
        match self {
            Self::Value(val) => Some(val),
            Self::Global(_) => None,
        }
    }

    /// Resolves the property to its specific value if it is set, or returns an error if it is a global value.
    pub(crate) fn resolve(property: &Self) -> Result<&T, String> {
        match property {
            Self::Value(val) => Ok(val),
            Self::Global(global) => Err(format!("Cannot resolve global property: {global:?}")),
        }
    }

    /// Resolves the property to its specific value if it is set, or computes the value based on the global value and the provided context (parent and initial values).
    pub(crate) const fn resolve_with_context<'css>(&'css self, parent: &'css T, initial: &'css T) -> &'css T {
        match self {
            Self::Global(global) => match global {
                Global::Initial => initial,
                Global::Inherit => parent,
                Global::Unset => parent,
                Global::Revert | Global::RevertLayer => initial,
            },
            Self::Value(val) => val,
        }
    }

    pub(crate) fn compute(self, parent: T) -> T
    where
        T: Default,
    {
        match self {
            Self::Global(global) => match global {
                Global::Inherit | Global::Unset => parent,
                Global::Initial | Global::Revert | Global::RevertLayer => T::default(),
            },
            Self::Value(val) => val,
        }
    }

    /// Updates the property from a `&mut ComponentValueStream`. It first checks
    /// if the value is a global value, and if so, updates the property accordingly.
    /// If not, it tries to parse the stream into the specific type T and updates
    /// the property with the parsed value.
    pub(crate) fn update_property(property: &mut Self, stream: &mut ComponentValueStream) -> Result<(), CssValueError> {
        let checkpoint = stream.checkpoint();

        if let Ok(global) = Global::parse(stream) {
            *property = Self::Global(global);
            return Ok(());
        }

        stream.restore(checkpoint);
        *property = Self::from(T::parse(stream)?);
        Ok(())
    }
}

impl<T> From<T> for CSSProperty<T> {
    fn from(value: T) -> Self {
        Self::Value(value)
    }
}

// Background
pub type BackgroundAttachmentProperty = CSSProperty<BackgroundAttachment>;
pub type BackgroundBlendModeProperty = CSSProperty<BackgroundBlendMode>;
pub type BackgroundClipProperty = CSSProperty<BackgroundClip>;
pub type BackgroundImageProperty = CSSProperty<BackgroundImage>;
pub type BackgroundOriginProperty = CSSProperty<BackgroundOrigin>;
pub type BackgroundRepeatProperty = CSSProperty<BackgroundRepeat>;
pub type BackgroundPositionXProperty = CSSProperty<BackgroundPositionX>;
pub type BackgroundPositionYProperty = CSSProperty<BackgroundPositionY>;
pub type BackgroundSizeProperty = CSSProperty<BackgroundSize>;

// Border
pub type BorderWidthValueProperty = CSSProperty<BorderWidth>;
pub type BorderStyleValueProperty = CSSProperty<BorderStyle>;

// Color
pub type ColorProperty = CSSProperty<Color>;

// Dimensions
pub type SizeProperty = CSSProperty<Size>;
pub type MaxSizeProperty = CSSProperty<MaxSize>;

// Display
pub type ClearProperty = CSSProperty<Clear>;
pub type DisplayProperty = CSSProperty<Display>;
pub type FloatProperty = CSSProperty<Float>;

// Flex & Grid
pub type FlexBasisProperty = CSSProperty<FlexBasis>;
pub type FlexDirectionProperty = CSSProperty<FlexDirection>;
pub type FlexValueProperty = CSSProperty<Flex>;
pub type FlexWrapProperty = CSSProperty<FlexWrap>;
pub type OrderProperty = CSSProperty<Order>;

// Font
pub type FontWeightProperty = CSSProperty<FontWeight>;
pub type FontFamilyProperty = CSSProperty<FontFamily>;
pub type FontSizeProperty = CSSProperty<FontSize>;

// Margin & Padding
pub type MarginProperty = CSSProperty<MarginValue>;
pub type OffsetProperty = CSSProperty<OffsetValue>;

// Position
pub type PositionProperty = CSSProperty<Position>;

// Text
pub type LineHeightProperty = CSSProperty<LineHeight>;
pub type TextAlignProperty = CSSProperty<TextAlign>;
pub type WritingModeProperty = CSSProperty<WritingMode>;
pub type WhitespaceProperty = CSSProperty<Whitespace>;

// Misc
pub type CursorProperty = CSSProperty<Cursor>;
