use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

use crate::{
    background::{Attachment, BgClip, Clip, VisualBox},
    blend::BlendMode,
    image::Image,
    length::{Length, LengthUnit},
    percentage::{LengthPercentage, Percentage},
    position::{Center, PositionX, PositionY},
};

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundAttachment(pub Vec<Attachment>);

impl Default for BackgroundAttachment {
    fn default() -> Self {
        Self(vec![Attachment::Scroll])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundAttachment {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut attachments = Vec::new();

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => match ident.as_str() {
                        "scroll" => attachments.push(Attachment::Scroll),
                        "fixed" => attachments.push(Attachment::Fixed),
                        "local" => attachments.push(Attachment::Local),
                        _ => continue,
                    },
                    _ => continue,
                }
            }
        }

        if attachments.is_empty() {
            Err(format!("No valid Attachment found for BackgroundAttachment: {:?}", value))
        } else {
            Ok(Self(attachments))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundBlendMode(pub Vec<BlendMode>);

impl Default for BackgroundBlendMode {
    fn default() -> Self {
        Self(vec![BlendMode::Normal])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundBlendMode {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut modes = Vec::new();
        for cv in value {
            if let ComponentValue::Token(token) = cv
                && let CssTokenKind::Ident(ident) = &token.kind
                && let Ok(mode) = ident.parse::<BlendMode>()
            {
                modes.push(mode);
            }
        }

        if modes.is_empty() {
            Err(format!("No valid BlendMode found for BackgroundBlendMode: {:?}", value))
        } else {
            Ok(Self(modes))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundClip(pub Vec<BgClip>);

impl Default for BackgroundClip {
    fn default() -> Self {
        Self(vec![BgClip::Visual(VisualBox::Border)])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundClip {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut clips = Vec::new();
        let mut clip_values = Vec::with_capacity(2);

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => match ident.as_str() {
                        "text" => clip_values.push(Clip::Text),
                        "border-area" => clip_values.push(Clip::BorderArea),
                        "border-box" => clips.push(BgClip::Visual(VisualBox::Border)),
                        "padding-box" => clips.push(BgClip::Visual(VisualBox::Padding)),
                        "content-box" => clips.push(BgClip::Visual(VisualBox::Content)),
                        _ => continue,
                    },
                    CssTokenKind::Comma => {
                        if !clip_values.is_empty() {
                            let first_clip = clip_values[0];
                            let second_clip = clip_values.get(1).cloned();
                            clips.push(BgClip::Clip(first_clip, second_clip));
                            clip_values.clear();
                        }
                    }
                    _ => continue,
                }
            }
        }

        if !clip_values.is_empty() {
            let first_clip = clip_values[0];
            let second_clip = clip_values.get(1).cloned();
            clips.push(BgClip::Clip(first_clip, second_clip));
        }

        if clips.is_empty() {
            Err(format!("No valid BgClip found for BackgroundClip: {:?}", value))
        } else {
            Ok(Self(clips))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundOrigin(pub Vec<VisualBox>);

impl Default for BackgroundOrigin {
    fn default() -> Self {
        Self(vec![VisualBox::Padding])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundOrigin {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut origins = Vec::new();

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => match ident.as_str() {
                        "content-box" => origins.push(VisualBox::Content),
                        "padding-box" => origins.push(VisualBox::Padding),
                        "border-box" => origins.push(VisualBox::Border),
                        _ => continue,
                    },
                    _ => continue,
                }
            }
        }

        if origins.is_empty() {
            Err(format!("No valid VisualBox found for BackgroundOrigin: {:?}", value))
        } else {
            Ok(Self(origins))
        }
    }
}

#[derive(Debug, Clone, Copy, EnumString, PartialEq, Eq)]
#[strum(serialize_all = "kebab-case", ascii_case_insensitive)]
pub enum RepeatStyle {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundRepeat(pub Vec<(RepeatStyle, RepeatStyle)>);

impl Default for BackgroundRepeat {
    fn default() -> Self {
        Self(vec![(RepeatStyle::Repeat, RepeatStyle::Repeat)])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundRepeat {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut keywords = Vec::new();

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("repeat") {
                            keywords.push((RepeatStyle::Repeat, RepeatStyle::Repeat));
                        } else if ident.eq_ignore_ascii_case("space") {
                            keywords.push((RepeatStyle::Space, RepeatStyle::Space));
                        } else if ident.eq_ignore_ascii_case("round") {
                            keywords.push((RepeatStyle::Round, RepeatStyle::Round));
                        } else if ident.eq_ignore_ascii_case("no-repeat") {
                            keywords.push((RepeatStyle::NoRepeat, RepeatStyle::NoRepeat));
                        } else if ident.eq_ignore_ascii_case("repeat-x") {
                            keywords.push((RepeatStyle::Repeat, RepeatStyle::NoRepeat));
                        } else if ident.eq_ignore_ascii_case("repeat-y") {
                            keywords.push((RepeatStyle::NoRepeat, RepeatStyle::Repeat));
                        }
                    }
                    _ => continue,
                }
            }
        }

        if keywords.is_empty() {
            Err(format!("No valid RepeatStyle pairs found for BackgroundRepeat: {:?}", value))
        } else {
            Ok(Self(keywords))
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundPositionX(pub Vec<PositionX>);

impl Default for BackgroundPositionX {
    fn default() -> Self {
        Self(vec![PositionX::Relative((
            None,
            Some(LengthPercentage::Percentage(Percentage::new(0.0))),
        ))])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundPositionY(pub Vec<PositionY>);

impl Default for BackgroundPositionY {
    fn default() -> Self {
        Self(vec![PositionY::Relative((
            None,
            Some(LengthPercentage::Percentage(Percentage::new(0.0))),
        ))])
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundPositionX {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut positions: Vec<PositionX> = Vec::new();
        let mut horizontal_side = None;
        let mut length_percentage = None;

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("center") {
                            if horizontal_side.is_some() {
                                positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                            } else if length_percentage.is_some() {
                                positions.push(PositionX::Center(Center::Center, length_percentage.take()));
                            }
                        } else if let Ok(h) = ident.parse() {
                            if horizontal_side.is_some() || length_percentage.is_some() {
                                positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                            }
                            horizontal_side = Some(h);
                        } else {
                            return Err(format!("Invalid horizontal side keyword: '{}'", ident));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| "Invalid length unit".to_string())?;
                        if length_percentage.is_some() {
                            return Err("Duplicate length or percentage".to_string());
                        }
                        length_percentage =
                            Some(LengthPercentage::Length(Length::new(value.to_f64() as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(value) => {
                        if length_percentage.is_some() {
                            return Err("Duplicate length or percentage".to_string());
                        }
                        length_percentage = Some(LengthPercentage::Percentage(Percentage::new(value.to_f64() as f32)));
                    }
                    _ => continue,
                }
            }
        }

        if horizontal_side.is_some() || length_percentage.is_some() {
            positions.push(PositionX::Relative((horizontal_side, length_percentage)));
        }

        if positions.is_empty() {
            Err(format!("No valid PositionX found for BackgroundPositionX: {:?}", value))
        } else {
            Ok(Self(positions))
        }
    }
}

impl TryFrom<&[ComponentValue]> for BackgroundPositionY {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut positions: Vec<PositionY> = Vec::new();
        let mut vertical_side = None;
        let mut length_percentage = None;

        for cv in value {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("center") {
                            if vertical_side.is_some() {
                                positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                            } else if length_percentage.is_some() {
                                positions.push(PositionY::Center(Center::Center, length_percentage.take()));
                            }
                        } else if let Ok(v) = ident.parse() {
                            if vertical_side.is_some() || length_percentage.is_some() {
                                positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                            }
                            vertical_side = Some(v);
                        } else {
                            return Err(format!("Invalid vertical side keyword: '{}'", ident));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| "Invalid length unit".to_string())?;
                        if length_percentage.is_some() {
                            return Err("Duplicate length or percentage".to_string());
                        }
                        length_percentage =
                            Some(LengthPercentage::Length(Length::new(value.to_f64() as f32, len_unit)));
                    }
                    CssTokenKind::Percentage(value) => {
                        if length_percentage.is_some() {
                            return Err("Duplicate length or percentage".to_string());
                        }
                        length_percentage = Some(LengthPercentage::Percentage(Percentage::new(value.to_f64() as f32)));
                    }
                    _ => continue,
                }
            }
        }

        if vertical_side.is_some() || length_percentage.is_some() {
            positions.push(PositionY::Relative((vertical_side, length_percentage)));
        }

        if positions.is_empty() {
            Err(format!("No valid PositionY found for BackgroundPositionY: {:?}", value))
        } else {
            Ok(Self(positions))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundImage(pub Vec<Image>);

impl TryFrom<&[ComponentValue]> for BackgroundImage {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let mut images = Vec::new();

        for cv in value {
            if let ComponentValue::Function(func) = cv
                && let Ok(image) = Image::try_from(func)
            {
                images.push(image);
            }
        }

        if images.is_empty() {
            Err(format!("No valid Image found for BackgroundImage: {:?}", value))
        } else {
            Ok(Self(images))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WidthHeightSize {
    Auto,
    Length(LengthPercentage),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Cover,
    Contain,
    WidthHeight(WidthHeightSize, Option<WidthHeightSize>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundSize(pub Vec<Size>);

impl TryFrom<&[ComponentValue]> for BackgroundSize {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let values = value.split(|cv| matches!(cv, ComponentValue::Token(token) if token.kind == CssTokenKind::Comma));
        let mut sizes = Vec::new();
        let mut width_height_values = Vec::with_capacity(2);

        for group in values {
            let mut size_pair = Vec::new();
            for cv in group {
                if let ComponentValue::Token(token) = cv {
                    match &token.kind {
                        CssTokenKind::Ident(ident) => {
                            if ident.eq_ignore_ascii_case("auto") {
                                width_height_values.push(WidthHeightSize::Auto);
                            } else if ident.eq_ignore_ascii_case("cover") {
                                if !width_height_values.is_empty() {
                                    let values = std::mem::take(&mut width_height_values);
                                    size_pair.push(Size::WidthHeight(values[0], values.get(1).cloned()));
                                }

                                size_pair.push(Size::Cover);
                            } else if ident.eq_ignore_ascii_case("contain") {
                                if !width_height_values.is_empty() {
                                    let values = std::mem::take(&mut width_height_values);
                                    size_pair.push(Size::WidthHeight(values[0], values.get(1).cloned()));
                                }

                                size_pair.push(Size::Contain);
                            } else {
                                continue;
                            }
                        }
                        CssTokenKind::Dimension { value, unit } => {
                            let len_unit = unit
                                .parse::<LengthUnit>()
                                .map_err(|_| "Invalid length unit".to_string())?;
                            width_height_values.push(WidthHeightSize::Length(LengthPercentage::Length(Length::new(
                                value.to_f64() as f32,
                                len_unit,
                            ))));
                        }
                        CssTokenKind::Percentage(value) => {
                            width_height_values.push(WidthHeightSize::Length(LengthPercentage::Percentage(
                                Percentage::new(value.to_f64() as f32),
                            )));
                        }
                        _ => continue,
                    }
                }
            }

            if !size_pair.is_empty() {
                sizes.extend(size_pair);
            } else if !width_height_values.is_empty() {
                if width_height_values.len() == 1 && width_height_values[0] == WidthHeightSize::Auto {
                    sizes.push(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)));
                } else {
                    let values = std::mem::take(&mut width_height_values);
                    sizes.push(Size::WidthHeight(values[0], values.get(1).cloned()));
                }
            }
        }

        if sizes.is_empty() {
            Err(format!("No valid Size pairs found for BackgroundSize: {:?}", value))
        } else {
            Ok(Self(sizes))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::position::{HorizontalOrXSide, HorizontalSide, VerticalOrYSide, VerticalSide};
    use css_cssom::CSSStyleSheet;

    /// Helper: parse an inline CSS declaration and return the component values
    /// for the first declaration found.
    fn parse_value(css: &str) -> Vec<css_cssom::ComponentValue> {
        let decls = CSSStyleSheet::from_inline(css);
        assert!(!decls.is_empty(), "No declarations parsed from: {css}");
        decls[0].original_values.clone()
    }

    #[test]
    fn attachment_scroll() {
        let cvs = parse_value("background-attachment: scroll");
        let att = BackgroundAttachment::try_from(cvs.as_slice()).unwrap();
        assert_eq!(att.0, vec![Attachment::Scroll]);
    }

    #[test]
    fn attachment_fixed() {
        let cvs = parse_value("background-attachment: fixed");
        let att = BackgroundAttachment::try_from(cvs.as_slice()).unwrap();
        assert_eq!(att.0, vec![Attachment::Fixed]);
    }

    #[test]
    fn attachment_local() {
        let cvs = parse_value("background-attachment: local");
        let att = BackgroundAttachment::try_from(cvs.as_slice()).unwrap();
        assert_eq!(att.0, vec![Attachment::Local]);
    }

    #[test]
    fn attachment_invalid() {
        let cvs = parse_value("background-attachment: banana");
        assert!(BackgroundAttachment::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn repeat_single_repeat() {
        let cvs = parse_value("background-repeat: repeat");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::Repeat);
        assert_eq!(rep.0[1].0, RepeatStyle::Repeat);
    }

    #[test]
    fn repeat_no_repeat() {
        let cvs = parse_value("background-repeat: no-repeat");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::NoRepeat);
        assert_eq!(rep.0[1].0, RepeatStyle::NoRepeat);
    }

    #[test]
    fn repeat_repeat_x() {
        let cvs = parse_value("background-repeat: repeat-x");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::Repeat);
        assert_eq!(rep.0[1].0, RepeatStyle::NoRepeat);
    }

    #[test]
    fn repeat_repeat_y() {
        let cvs = parse_value("background-repeat: repeat-y");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::NoRepeat);
        assert_eq!(rep.0[1].0, RepeatStyle::Repeat);
    }

    #[test]
    fn repeat_two_values() {
        let cvs = parse_value("background-repeat: round space");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::Round);
        assert_eq!(rep.0[1].0, RepeatStyle::Space);
    }

    #[test]
    fn repeat_invalid() {
        let cvs = parse_value("background-repeat: banana");
        assert!(BackgroundRepeat::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn origin_padding_box() {
        let cvs = parse_value("background-origin: padding-box");
        let orig = BackgroundOrigin::try_from(cvs.as_slice()).unwrap();
        assert_eq!(orig.0, vec![VisualBox::Padding]);
    }

    #[test]
    fn origin_content_box() {
        let cvs = parse_value("background-origin: content-box");
        let orig = BackgroundOrigin::try_from(cvs.as_slice()).unwrap();
        assert_eq!(orig.0, vec![VisualBox::Content]);
    }

    #[test]
    fn origin_border_box() {
        let cvs = parse_value("background-origin: border-box");
        let orig = BackgroundOrigin::try_from(cvs.as_slice()).unwrap();
        assert_eq!(orig.0, vec![VisualBox::Border]);
    }

    #[test]
    fn origin_invalid() {
        let cvs = parse_value("background-origin: banana");
        assert!(BackgroundOrigin::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn clip_border_box() {
        let cvs = parse_value("background-clip: border-box");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(clip.0, vec![BgClip::Visual(VisualBox::Border)]);
    }

    #[test]
    fn clip_padding_box() {
        let cvs = parse_value("background-clip: padding-box");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(clip.0, vec![BgClip::Visual(VisualBox::Padding)]);
    }

    #[test]
    fn clip_content_box() {
        let cvs = parse_value("background-clip: content-box");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(clip.0, vec![BgClip::Visual(VisualBox::Content)]);
    }

    #[test]
    fn clip_text() {
        let cvs = parse_value("background-clip: text");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(clip.0, vec![BgClip::Clip(Clip::Text, None)]);
    }

    #[test]
    fn clip_invalid() {
        let cvs = parse_value("background-clip: banana");
        assert!(BackgroundClip::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn size_cover() {
        let cvs = parse_value("background-size: cover");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(sz.0, vec![Size::Cover]);
    }

    #[test]
    fn size_contain() {
        let cvs = parse_value("background-size: contain");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(sz.0, vec![Size::Contain]);
    }

    #[test]
    fn size_auto() {
        let cvs = parse_value("background-size: auto");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![Size::WidthHeight(
                WidthHeightSize::Auto,
                Some(WidthHeightSize::Auto)
            )]
        );
    }

    #[test]
    fn size_length() {
        let cvs = parse_value("background-size: 100px");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![Size::WidthHeight(
                WidthHeightSize::Length(LengthPercentage::Length(Length::new(100.0, LengthUnit::Px))),
                None
            )]
        );
    }

    #[test]
    fn size_percentage() {
        let cvs = parse_value("background-size: 50%");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![Size::WidthHeight(
                WidthHeightSize::Length(LengthPercentage::Percentage(Percentage::new(50.0))),
                None
            )]
        );
    }

    #[test]
    fn size_two_values() {
        let cvs = parse_value("background-size: 100px 50%");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![Size::WidthHeight(
                WidthHeightSize::Length(LengthPercentage::Length(Length::new(100.0, LengthUnit::Px))),
                Some(WidthHeightSize::Length(LengthPercentage::Percentage(Percentage::new(50.0))))
            )]
        );
    }

    #[test]
    fn size_auto_auto() {
        let cvs = parse_value("background-size: auto auto");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![Size::WidthHeight(
                WidthHeightSize::Auto,
                Some(WidthHeightSize::Auto)
            )]
        );
    }

    #[test]
    fn image_url() {
        let cvs = parse_value("background-image: url('test.png')");
        let img = BackgroundImage::try_from(cvs.as_slice()).unwrap();
        assert_eq!(img.0.len(), 1);
        assert!(matches!(&img.0[0], Image::Url(s) if s == "test.png"));
    }

    #[test]
    fn image_linear_gradient() {
        let cvs = parse_value("background-image: linear-gradient(red, blue)");
        let img = BackgroundImage::try_from(cvs.as_slice()).unwrap();
        assert_eq!(img.0.len(), 1);
        assert!(matches!(&img.0[0], Image::Gradient(_)));
    }

    #[test]
    fn image_invalid() {
        let cvs = parse_value("background-image: banana");
        assert!(BackgroundImage::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn position_x_left() {
        let cvs = parse_value("background-position-x: left");
        let pos = BackgroundPositionX::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
    }

    #[test]
    fn position_x_center() {
        let cvs = parse_value("background-position-x: center");
        let pos = BackgroundPositionX::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
        assert!(matches!(pos.0[0], PositionX::Center(_, _)));
    }

    #[test]
    fn position_x_percentage() {
        let cvs = parse_value("background-position-x: 50%");
        let pos = BackgroundPositionX::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
    }

    #[test]
    fn position_y_top() {
        let cvs = parse_value("background-position-y: top");
        let pos = BackgroundPositionY::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
    }

    #[test]
    fn position_y_center() {
        let cvs = parse_value("background-position-y: center");
        let pos = BackgroundPositionY::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
        assert!(matches!(pos.0[0], PositionY::Center(_, _)));
    }

    #[test]
    fn position_y_percentage() {
        let cvs = parse_value("background-position-y: 25%");
        let pos = BackgroundPositionY::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
    }

    #[test]
    fn attachment_multiple_values() {
        let cvs = parse_value("background-attachment: scroll, fixed, local");
        let att = BackgroundAttachment::try_from(cvs.as_slice()).unwrap();
        assert_eq!(att.0, vec![Attachment::Scroll, Attachment::Fixed, Attachment::Local]);
    }

    #[test]
    fn repeat_case_insensitive() {
        let cvs = parse_value("background-repeat: REPEAT");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::Repeat);
        assert_eq!(rep.0[1].0, RepeatStyle::Repeat);
    }

    #[test]
    fn repeat_two_values_with_no_repeat() {
        let cvs = parse_value("background-repeat: repeat no-repeat");
        let rep = BackgroundRepeat::try_from(cvs.as_slice()).unwrap();
        assert_eq!(rep.0[0].0, RepeatStyle::Repeat);
        assert_eq!(rep.0[1].0, RepeatStyle::NoRepeat);
    }

    #[test]
    fn origin_multiple_values() {
        let cvs = parse_value("background-origin: padding-box content-box");
        let orig = BackgroundOrigin::try_from(cvs.as_slice()).unwrap();
        assert_eq!(orig.0, vec![VisualBox::Padding, VisualBox::Content]);
    }

    #[test]
    fn clip_text_border_area_pair() {
        let cvs = parse_value("background-clip: text border-area");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(clip.0, vec![BgClip::Clip(Clip::Text, Some(Clip::BorderArea))]);
    }

    #[test]
    fn clip_multiple_groups() {
        let cvs = parse_value("background-clip: text, padding-box");
        let clip = BackgroundClip::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            clip.0,
            vec![
                BgClip::Clip(Clip::Text, None),
                BgClip::Visual(VisualBox::Padding)
            ]
        );
    }

    #[test]
    fn size_multiple_groups() {
        let cvs = parse_value("background-size: cover, 100px 50%");
        let sz = BackgroundSize::try_from(cvs.as_slice()).unwrap();
        assert_eq!(
            sz.0,
            vec![
                Size::Cover,
                Size::WidthHeight(
                    WidthHeightSize::Length(LengthPercentage::Length(Length::new(100.0, LengthUnit::Px))),
                    Some(WidthHeightSize::Length(LengthPercentage::Percentage(Percentage::new(50.0))))
                )
            ]
        );
    }

    #[test]
    fn size_invalid_three_values() {
        let cvs = parse_value("background-size: auto auto auto");
        assert!(BackgroundSize::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn image_multiple_urls() {
        let cvs = parse_value("background-image: url('a.png'), url('b.png')");
        let img = BackgroundImage::try_from(cvs.as_slice()).unwrap();
        assert_eq!(img.0.len(), 2);
    }

    #[test]
    fn position_x_left_with_offset() {
        let cvs = parse_value("background-position-x: left 10%");
        let pos = BackgroundPositionX::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
        assert!(matches!(
            pos.0[0],
            PositionX::Relative((
                Some(HorizontalOrXSide::Horizontal(HorizontalSide::Left)),
                Some(LengthPercentage::Percentage(p))
            )) if p == Percentage::new(10.0)
        ));
    }

    #[test]
    fn position_y_bottom_with_length() {
        let cvs = parse_value("background-position-y: bottom 12px");
        let pos = BackgroundPositionY::try_from(cvs.as_slice()).unwrap();
        assert_eq!(pos.0.len(), 1);
        assert!(matches!(
            pos.0[0],
            PositionY::Relative((
                Some(VerticalOrYSide::Vertical(VerticalSide::Bottom)),
                Some(LengthPercentage::Length(l))
            )) if l == Length::new(12.0, LengthUnit::Px)
        ));
    }

    #[test]
    fn position_x_duplicate_lengths_error() {
        let cvs = parse_value("background-position-x: 10% 20%");
        assert!(BackgroundPositionX::try_from(cvs.as_slice()).is_err());
    }

    #[test]
    fn position_y_duplicate_lengths_error() {
        let cvs = parse_value("background-position-y: 10% 20%");
        assert!(BackgroundPositionY::try_from(cvs.as_slice()).is_err());
    }
}
