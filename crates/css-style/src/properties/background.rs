use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};
use strum::EnumString;

use crate::{
    background::{Attachment, BgClip, Clip, VisualBox},
    blend::BlendMode,
    image::Image,
    length::{Length, LengthUnit},
    percentage::{LengthPercentage, Percentage},
    position::{Center, PositionX, PositionY},
    properties::CSSParsable,
};

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundAttachment(pub Vec<Attachment>);

impl Default for BackgroundAttachment {
    fn default() -> Self {
        Self(vec![Attachment::Scroll])
    }
}

impl CSSParsable for BackgroundAttachment {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut attachments = Vec::new();

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("scroll") {
                            attachments.push(Attachment::Scroll);
                        } else if ident.eq_ignore_ascii_case("fixed") {
                            attachments.push(Attachment::Fixed);
                        } else if ident.eq_ignore_ascii_case("local") {
                            attachments.push(Attachment::Local);
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                }
            }
        }

        if attachments.is_empty() {
            Err("No valid Attachment found for BackgroundAttachment".to_string())
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

impl CSSParsable for BackgroundBlendMode {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut modes = Vec::new();

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv
                && let CssTokenKind::Ident(ident) = &token.kind
                && let Ok(mode) = ident.parse::<BlendMode>()
            {
                modes.push(mode);
            }
        }

        if modes.is_empty() {
            Err("No valid BlendMode found for BackgroundBlendMode".to_string())
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

impl CSSParsable for BackgroundClip {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut clips = Vec::new();
        let mut clip_values = Vec::with_capacity(2);

        while let Some(cv) = stream.next_cv() {
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
            Err("No valid BgClip found for BackgroundClip".to_string())
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

impl CSSParsable for BackgroundOrigin {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut origins = Vec::new();

        while let Some(cv) = stream.next_cv() {
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
            Err("No valid VisualBox found for BackgroundOrigin".to_string())
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

impl CSSParsable for BackgroundRepeat {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut keywords = Vec::new();
        let mut current_pair = (None, None);

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("repeat") {
                            if current_pair.0.is_none() {
                                current_pair.0 = Some(RepeatStyle::Repeat);
                            } else if current_pair.1.is_none() {
                                current_pair.1 = Some(RepeatStyle::Repeat);
                            }
                        } else if ident.eq_ignore_ascii_case("space") {
                            if current_pair.0.is_none() {
                                current_pair.0 = Some(RepeatStyle::Space);
                            } else if current_pair.1.is_none() {
                                current_pair.1 = Some(RepeatStyle::Space);
                            }
                        } else if ident.eq_ignore_ascii_case("round") {
                            if current_pair.0.is_none() {
                                current_pair.0 = Some(RepeatStyle::Round);
                            } else if current_pair.1.is_none() {
                                current_pair.1 = Some(RepeatStyle::Round);
                            }
                        } else if ident.eq_ignore_ascii_case("no-repeat") {
                            if current_pair.0.is_none() {
                                current_pair.0 = Some(RepeatStyle::NoRepeat);
                            } else if current_pair.1.is_none() {
                                current_pair.1 = Some(RepeatStyle::NoRepeat);
                            }
                        } else if ident.eq_ignore_ascii_case("repeat-x") {
                            current_pair = (Some(RepeatStyle::Repeat), Some(RepeatStyle::NoRepeat));
                        } else if ident.eq_ignore_ascii_case("repeat-y") {
                            current_pair = (Some(RepeatStyle::NoRepeat), Some(RepeatStyle::Repeat));
                        }
                    }
                    CssTokenKind::Comma => {
                        if let (Some(first), Some(second)) = current_pair {
                            keywords.push((first, second));
                        } else if let (Some(first), None) = current_pair {
                            keywords.push((first, first));
                        }
                        current_pair = (None, None);
                    }
                    _ => continue,
                },
                _ => continue,
            }
        }

        if let (Some(first), Some(second)) = current_pair {
            keywords.push((first, second));
        } else if let (Some(first), None) = current_pair {
            keywords.push((first, first));
        }

        if keywords.is_empty() {
            Err("No valid RepeatStyle pairs found for BackgroundRepeat".to_string())
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

impl CSSParsable for BackgroundPositionX {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut positions = Vec::new();
        let mut horizontal_side = None;
        let mut length_percentage = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("center") {
                            if horizontal_side.is_some() {
                                positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                            } else if length_percentage.is_some() {
                                positions.push(PositionX::Center(Center::Center, length_percentage.take()));
                            } else {
                                positions.push(PositionX::Center(Center::Center, None));
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
                    CssTokenKind::Comma => {
                        if horizontal_side.is_some() || length_percentage.is_some() {
                            positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                        }
                    }
                    _ => continue,
                }
            }
        }

        if horizontal_side.is_some() || length_percentage.is_some() {
            positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
        }

        if positions.is_empty() {
            Err("No valid PositionX found for BackgroundPositionX.".to_string())
        } else {
            Ok(Self(positions))
        }
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

impl CSSParsable for BackgroundPositionY {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut positions = Vec::new();
        let mut vertical_side = None;
        let mut length_percentage = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if ident.eq_ignore_ascii_case("center") {
                            if vertical_side.is_some() {
                                positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                            } else if length_percentage.is_some() {
                                positions.push(PositionY::Center(Center::Center, length_percentage.take()));
                            } else {
                                positions.push(PositionY::Center(Center::Center, None));
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
                    CssTokenKind::Comma => {
                        if vertical_side.is_some() || length_percentage.is_some() {
                            positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                        }
                    }
                    _ => continue,
                }
            }
        }

        if vertical_side.is_some() || length_percentage.is_some() {
            positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
        }

        if positions.is_empty() {
            Err("No valid PositionY found for BackgroundPositionY.".to_string())
        } else {
            Ok(Self(positions))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundImage(pub Vec<Image>);

impl CSSParsable for BackgroundImage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut images = Vec::new();

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("none") => {
                        images.push(Image::None);
                    }
                    CssTokenKind::Url(url) => images.push(Image::Url(url.clone())),
                    _ => continue,
                },
                ComponentValue::Function(func) => match Image::try_from(func) {
                    Ok(img) => images.push(img),
                    Err(e) => return Err(format!("Failed to parse image function '{}': {}", func.name, e)),
                },
                _ => continue,
            }
        }

        if images.is_empty() {
            Err("No valid Image found for BackgroundImage".to_string())
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

impl Default for Size {
    fn default() -> Self {
        Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BackgroundSize(pub Vec<Size>);

impl Default for BackgroundSize {
    fn default() -> Self {
        Self(vec![(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)))])
    }
}

impl CSSParsable for BackgroundSize {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, String> {
        stream.skip_whitespace();
        let mut sizes = Vec::new();
        let mut width_height_values = Vec::with_capacity(2);
        let mut size_pair = Vec::new();

        // TODO: calc();

        while let Some(cv) = stream.next_cv() {
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
                    CssTokenKind::Comma => {
                        if !size_pair.is_empty() {
                            sizes.append(&mut size_pair)
                        } else if width_height_values.len() > 2 {
                            return Err("Too many width/height values".to_string());
                        } else if !width_height_values.is_empty() {
                            if width_height_values.len() == 1 && width_height_values[0] == WidthHeightSize::Auto {
                                sizes.push(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)));
                            } else {
                                let values = std::mem::take(&mut width_height_values);
                                sizes.push(Size::WidthHeight(values[0], values.get(1).cloned()));
                            }
                        }
                    }
                    _ => continue,
                }
            }
        }

        if !size_pair.is_empty() {
            sizes.append(&mut size_pair);
        } else if width_height_values.len() > 2 {
            return Err("Too many width/height values".to_string());
        } else if !width_height_values.is_empty() {
            if width_height_values.len() == 1 && width_height_values[0] == WidthHeightSize::Auto {
                sizes.push(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)));
            } else {
                let values = std::mem::take(&mut width_height_values);
                sizes.push(Size::WidthHeight(values[0], values.get(1).cloned()));
            }
        }

        if sizes.is_empty() {
            Err(format!("No valid Size pairs found for BackgroundSize: {:?}", sizes))
        } else {
            Ok(Self(sizes))
        }
    }
}
