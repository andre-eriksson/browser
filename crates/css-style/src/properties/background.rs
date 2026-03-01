use css_cssom::{ComponentValue, CssTokenKind};
use strum::EnumString;

use crate::{
    background::{Attachment, BgClip, Clip, VisualBox},
    image::Image,
    length::{Length, LengthUnit},
    percentage::{LengthPercentage, Percentage},
    position::{Center, PositionX, PositionY},
};

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundAttachment {
    pub attachments: Vec<Attachment>,
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
            Ok(Self { attachments })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundClip {
    pub clips: Vec<BgClip>,
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
            Ok(Self { clips })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundOrigin {
    pub origins: Vec<VisualBox>,
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
            Ok(Self { origins })
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BackgroundRepeat {
    pub horizontal: RepeatStyle,
    pub vertical: RepeatStyle,
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
                            keywords.push(RepeatStyle::Repeat);
                        } else if ident.eq_ignore_ascii_case("space") {
                            keywords.push(RepeatStyle::Space);
                        } else if ident.eq_ignore_ascii_case("round") {
                            keywords.push(RepeatStyle::Round);
                        } else if ident.eq_ignore_ascii_case("no-repeat") {
                            keywords.push(RepeatStyle::NoRepeat);
                        } else if ident.eq_ignore_ascii_case("repeat-x") {
                            return Ok(Self {
                                horizontal: RepeatStyle::Repeat,
                                vertical: RepeatStyle::NoRepeat,
                            });
                        } else if ident.eq_ignore_ascii_case("repeat-y") {
                            return Ok(Self {
                                horizontal: RepeatStyle::NoRepeat,
                                vertical: RepeatStyle::Repeat,
                            });
                        }
                    }
                    _ => continue,
                }
            }
        }

        if keywords.is_empty() {
            Err(format!("No valid BackgroundRepeatKeyword found for BackgroundRepeat: {:?}", value))
        } else if keywords.len() == 1 {
            Ok(Self {
                horizontal: keywords[0],
                vertical: keywords[0],
            })
        } else {
            Ok(Self {
                horizontal: keywords[0],
                vertical: keywords[1],
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundPositionX(pub Vec<PositionX>);

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundPositionY(pub Vec<PositionY>);

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
                            if horizontal_side.is_some() || length_percentage.is_some() {
                                positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                            }

                            positions.push(PositionX::Center(Center::Center));
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
                            if vertical_side.is_some() || length_percentage.is_some() {
                                positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                            }

                            positions.push(PositionY::Center(Center::Center));
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

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundImage {
    images: Vec<Image>,
}

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
            Ok(Self { images })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Size {
    Auto,
    Cover,
    Contain,
    Length(Length),
    Percentage(Percentage),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundSize {
    pub sizes: Vec<(Size, Size)>,
}

impl TryFrom<&[ComponentValue]> for BackgroundSize {
    type Error = String;

    fn try_from(value: &[ComponentValue]) -> Result<Self, Self::Error> {
        let values = value.split(|cv| matches!(cv, ComponentValue::Token(token) if token.kind == CssTokenKind::Comma));

        let mut sizes = Vec::new();

        for group in values {
            let mut size_pair = Vec::new();
            for cv in group {
                if let ComponentValue::Token(token) = cv {
                    match &token.kind {
                        CssTokenKind::Ident(ident) => {
                            if ident.eq_ignore_ascii_case("auto") {
                                size_pair.push(Size::Auto);
                            } else if ident.eq_ignore_ascii_case("cover") {
                                size_pair.push(Size::Cover);
                            } else if ident.eq_ignore_ascii_case("contain") {
                                size_pair.push(Size::Contain);
                            } else {
                                continue;
                            }
                        }
                        CssTokenKind::Dimension { value, unit } => {
                            let len_unit = unit
                                .parse::<LengthUnit>()
                                .map_err(|_| "Invalid length unit".to_string())?;
                            size_pair.push(Size::Length(Length::new(value.to_f64() as f32, len_unit)));
                        }
                        CssTokenKind::Percentage(value) => {
                            size_pair.push(Size::Percentage(Percentage::new(value.to_f64() as f32)));
                        }
                        _ => continue,
                    }
                }
            }

            if size_pair.len() == 1 {
                sizes.push((size_pair[0], Size::Auto));
            } else if size_pair.len() == 2 {
                sizes.push((size_pair[0], size_pair[1]));
            } else {
                return Err(format!("Invalid number of values in BackgroundSize group: {:?}", group));
            }
        }

        if sizes.is_empty() {
            Err(format!("No valid Size pairs found for BackgroundSize: {:?}", value))
        } else {
            Ok(Self { sizes })
        }
    }
}
