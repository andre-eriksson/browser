use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use css_values::{
    CSSParsable,
    background::{Attachment, BgClip, BlendMode, Clip, RepeatStyle, Size, VisualBox, WidthHeightSize},
    combination::LengthPercentage,
    error::CssValueError,
    image::Image,
    numeric::Percentage,
    position::{
        BgPosition, BlockAxis, Center, HorizontalOrXSide, HorizontalSide, InlineAxis, PositionFour, PositionOne,
        PositionThree, PositionTwo, PositionX, PositionY, RelativeAxis, RelativeHorizontalSide, RelativeVerticalSide,
        Side, VerticalOrYSide, VerticalSide, XAxis, XAxisOrLengthPercentage, XSide, YAxis, YAxisOrLengthPercentage,
        YSide,
    },
    quantity::{Length, LengthUnit},
    text::WritingMode,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundAttachment(pub Vec<Attachment>);

impl Default for BackgroundAttachment {
    fn default() -> Self {
        Self(vec![Attachment::Scroll])
    }
}

impl CSSParsable for BackgroundAttachment {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut attachments = Vec::new();
        let mut current_attachment = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_attachment.is_some() {
                            return Err(CssValueError::InvalidValue(
                                "Multiple attachment keywords without a comma".into(),
                            ));
                        } else if ident.eq_ignore_ascii_case("scroll") {
                            current_attachment = Some(Attachment::Scroll);
                        } else if ident.eq_ignore_ascii_case("fixed") {
                            current_attachment = Some(Attachment::Fixed);
                        } else if ident.eq_ignore_ascii_case("local") {
                            current_attachment = Some(Attachment::Local);
                        }
                    }
                    CssTokenKind::Comma => {
                        if let Some(attachment) = current_attachment.take() {
                            attachments.push(attachment);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(attachment) = current_attachment.take() {
            attachments.push(attachment);
        }

        if attachments.is_empty() {
            Err(CssValueError::InvalidValue("No valid Attachment found for BackgroundAttachment".into()))
        } else {
            Ok(Self(attachments))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundBlendMode(pub Vec<BlendMode>);

impl Default for BackgroundBlendMode {
    fn default() -> Self {
        Self(vec![BlendMode::Normal])
    }
}

impl CSSParsable for BackgroundBlendMode {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut modes = Vec::new();
        let mut current_mode = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_mode.is_some() {
                            return Err(CssValueError::InvalidValue(
                                "Multiple blend mode keywords without a comma".into(),
                            ));
                        } else if let Ok(mode) = ident.parse() {
                            current_mode = Some(mode);
                        }
                    }
                    CssTokenKind::Comma => {
                        if let Some(mode) = current_mode.take() {
                            modes.push(mode);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(mode) = current_mode.take() {
            modes.push(mode);
        }

        if modes.is_empty() {
            Err(CssValueError::InvalidValue("No valid BlendMode found for BackgroundBlendMode".into()))
        } else {
            Ok(Self(modes))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundClip(pub Vec<BgClip>);

impl Default for BackgroundClip {
    fn default() -> Self {
        Self(vec![BgClip::Visual(VisualBox::Border)])
    }
}

impl CSSParsable for BackgroundClip {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut clips = Vec::new();
        let mut current_clip = None;
        let mut clip_values = Vec::with_capacity(2);

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_clip.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple clip keywords without a comma".into()));
                        } else if ident.eq_ignore_ascii_case("text") {
                            clip_values.push(Clip::Text);
                        } else if ident.eq_ignore_ascii_case("border-area") {
                            clip_values.push(Clip::BorderArea);
                        } else if ident.eq_ignore_ascii_case("border-box") {
                            current_clip = Some(BgClip::Visual(VisualBox::Border));
                        } else if ident.eq_ignore_ascii_case("padding-box") {
                            current_clip = Some(BgClip::Visual(VisualBox::Padding));
                        } else if ident.eq_ignore_ascii_case("content-box") {
                            current_clip = Some(BgClip::Visual(VisualBox::Content));
                        }
                    }
                    CssTokenKind::Comma => {
                        if let Some(clip) = current_clip.take() {
                            clips.push(clip);
                        } else if !clip_values.is_empty() {
                            let first_clip = clip_values[0];
                            let second_clip = clip_values.get(1).copied();
                            clips.push(BgClip::Clip(first_clip, second_clip));
                            clip_values.clear();
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(clip) = current_clip.take() {
            clips.push(clip);
        } else if !clip_values.is_empty() {
            let first_clip = clip_values[0];
            let second_clip = clip_values.get(1).copied();
            clips.push(BgClip::Clip(first_clip, second_clip));
            clip_values.clear();
        }

        if clips.is_empty() {
            Err(CssValueError::InvalidValue("No valid BgClip found for BackgroundClip".into()))
        } else {
            Ok(Self(clips))
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BackgroundImage(pub Vec<Image>);

impl CSSParsable for BackgroundImage {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut images = Vec::new();
        let mut current_image = None;

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(token) => match &token.kind {
                    CssTokenKind::Ident(ident) if ident.eq_ignore_ascii_case("none") => {
                        if current_image.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple image values without a comma".into()));
                        }

                        current_image = Some(Image::None);
                    }
                    CssTokenKind::Url(url) => {
                        if current_image.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple image values without a comma".into()));
                        }

                        current_image = Some(Image::Url(url.clone()));
                    }
                    CssTokenKind::Comma => {
                        if let Some(image) = current_image.take() {
                            images.push(image);
                        }
                    }
                    _ => {}
                },
                ComponentValue::Function(func) => match Image::try_from(func) {
                    Ok(img) => current_image = Some(img),
                    Err(e) => {
                        return Err(CssValueError::InvalidValue(format!(
                            "Failed to parse image function '{}': {}",
                            func.name, e
                        )));
                    }
                },
                ComponentValue::SimpleBlock(_) => {}
            }
        }

        if let Some(image) = current_image.take() {
            images.push(image);
        }

        if images.is_empty() {
            Err(CssValueError::InvalidValue("No valid Image found for BackgroundImage".into()))
        } else {
            Ok(Self(images))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundOrigin(pub Vec<VisualBox>);

impl Default for BackgroundOrigin {
    fn default() -> Self {
        Self(vec![VisualBox::Padding])
    }
}

impl CSSParsable for BackgroundOrigin {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut current_origin = None;
        let mut origins = Vec::new();

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_origin.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple origin keywords without a comma".into()));
                        } else if ident.eq_ignore_ascii_case("content-box") {
                            current_origin = Some(VisualBox::Content);
                        } else if ident.eq_ignore_ascii_case("padding-box") {
                            current_origin = Some(VisualBox::Padding);
                        } else if ident.eq_ignore_ascii_case("border-box") {
                            current_origin = Some(VisualBox::Border);
                        }
                    }
                    CssTokenKind::Comma => {
                        if let Some(origin) = current_origin.take() {
                            origins.push(origin);
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(origin) = current_origin.take() {
            origins.push(origin);
        }

        if origins.is_empty() {
            Err(CssValueError::InvalidValue("No valid VisualBox found for BackgroundOrigin".into()))
        } else {
            Ok(Self(origins))
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackgroundPosition(pub Vec<BgPosition>);

impl BackgroundPosition {
    /// Resolve a single `BgPosition` layer into `PositionX` / `PositionY` entries.
    pub(crate) fn resolve_bg_position_layer(
        position: BgPosition,
        writing_mode: WritingMode,
        x_pos: &mut Vec<PositionX>,
        y_pos: &mut Vec<PositionY>,
    ) {
        const fn resolve_horizontal_side(horizontal: HorizontalSide, writing_mode: WritingMode) -> HorizontalOrXSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => {
                    HorizontalOrXSide::Horizontal(horizontal)
                }
                WritingMode::VerticalRl | WritingMode::VerticalLr => match horizontal {
                    HorizontalSide::Left => HorizontalOrXSide::XSide(XSide::XStart),
                    HorizontalSide::Right => HorizontalOrXSide::XSide(XSide::XEnd),
                },
            }
        }

        const fn resolve_vertical_side(vertical: VerticalSide, writing_mode: WritingMode) -> VerticalOrYSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => match vertical {
                    VerticalSide::Top => VerticalOrYSide::YSide(YSide::YStart),
                    VerticalSide::Bottom => VerticalOrYSide::YSide(YSide::YEnd),
                },
                WritingMode::VerticalRl | WritingMode::VerticalLr => VerticalOrYSide::Vertical(vertical),
            }
        }

        const fn resolve_horizontal_x_side(side: Side, writing_mode: WritingMode) -> HorizontalOrXSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => match side {
                    Side::Start => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
                    Side::End => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
                },
                WritingMode::VerticalRl | WritingMode::VerticalLr => match side {
                    Side::Start => HorizontalOrXSide::XSide(XSide::XStart),
                    Side::End => HorizontalOrXSide::XSide(XSide::XEnd),
                },
            }
        }

        const fn resolve_vertical_y_side(side: Side, writing_mode: WritingMode) -> VerticalOrYSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => match side {
                    Side::Start => VerticalOrYSide::YSide(YSide::YStart),
                    Side::End => VerticalOrYSide::YSide(YSide::YEnd),
                },
                WritingMode::VerticalRl | WritingMode::VerticalLr => match side {
                    Side::Start => VerticalOrYSide::Vertical(VerticalSide::Top),
                    Side::End => VerticalOrYSide::Vertical(VerticalSide::Bottom),
                },
            }
        }

        const fn resolve_inline_axis(inline: InlineAxis, writing_mode: WritingMode) -> HorizontalOrXSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::VerticalRl | WritingMode::VerticalLr => match inline {
                    InlineAxis::InlineStart => HorizontalOrXSide::XSide(XSide::XStart),
                    InlineAxis::InlineEnd => HorizontalOrXSide::XSide(XSide::XEnd),
                },
                WritingMode::SidewaysLr => match inline {
                    InlineAxis::InlineStart => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
                    InlineAxis::InlineEnd => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
                },
                WritingMode::SidewaysRl => match inline {
                    InlineAxis::InlineStart => HorizontalOrXSide::Horizontal(HorizontalSide::Right),
                    InlineAxis::InlineEnd => HorizontalOrXSide::Horizontal(HorizontalSide::Left),
                },
            }
        }

        const fn resolve_block_axis(block: BlockAxis, writing_mode: WritingMode) -> VerticalOrYSide {
            match writing_mode {
                WritingMode::HorizontalTb | WritingMode::SidewaysLr | WritingMode::SidewaysRl => match block {
                    BlockAxis::BlockStart => VerticalOrYSide::YSide(YSide::YStart),
                    BlockAxis::BlockEnd => VerticalOrYSide::YSide(YSide::YEnd),
                },
                WritingMode::VerticalRl => match block {
                    BlockAxis::BlockStart => VerticalOrYSide::Vertical(VerticalSide::Top),
                    BlockAxis::BlockEnd => VerticalOrYSide::Vertical(VerticalSide::Bottom),
                },
                WritingMode::VerticalLr => match block {
                    BlockAxis::BlockStart => VerticalOrYSide::Vertical(VerticalSide::Bottom),
                    BlockAxis::BlockEnd => VerticalOrYSide::Vertical(VerticalSide::Top),
                },
            }
        }

        const fn resolve_x_side(x_side: XSide) -> PositionX {
            match x_side {
                XSide::XStart => PositionX::Relative((Some(HorizontalOrXSide::XSide(XSide::XStart)), None)),
                XSide::XEnd => PositionX::Relative((Some(HorizontalOrXSide::XSide(XSide::XEnd)), None)),
            }
        }

        const fn resolve_y_side(y_side: YSide) -> PositionY {
            match y_side {
                YSide::YStart => PositionY::Relative((Some(VerticalOrYSide::YSide(YSide::YStart)), None)),
                YSide::YEnd => PositionY::Relative((Some(VerticalOrYSide::YSide(YSide::YEnd)), None)),
            }
        }

        const fn resolve_x_axis(x_axis: XAxis) -> PositionX {
            match x_axis {
                XAxis::Center(center) => PositionX::Center(center, None),
                XAxis::Horizontal(horizontal) => {
                    PositionX::Relative((Some(HorizontalOrXSide::Horizontal(horizontal)), None))
                }
                XAxis::XSide(xside) => resolve_x_side(xside),
            }
        }

        const fn resolve_y_axis(y_axis: YAxis) -> PositionY {
            match y_axis {
                YAxis::Center(center) => PositionY::Center(center, None),
                YAxis::Vertical(vertical) => PositionY::Relative((Some(VerticalOrYSide::Vertical(vertical)), None)),
                YAxis::YSide(yside) => resolve_y_side(yside),
            }
        }

        match position {
            BgPosition::One(one) => match one {
                PositionOne::LengthPercentage(lp) => {
                    x_pos.push(PositionX::Relative((None, Some(lp.clone()))));
                    y_pos.push(PositionY::Relative((None, Some(lp))));
                }
                PositionOne::Horizontal(horizontal) => match horizontal {
                    HorizontalOrXSide::XSide(xside) => x_pos.push(resolve_x_side(xside)),
                    HorizontalOrXSide::Horizontal(h) => {
                        x_pos.push(PositionX::Relative((Some(HorizontalOrXSide::Horizontal(h)), None)));
                    }
                },
                PositionOne::Vertical(vertical) => match vertical {
                    VerticalOrYSide::YSide(yside) => y_pos.push(resolve_y_side(yside)),
                    VerticalOrYSide::Vertical(v) => {
                        y_pos.push(PositionY::Relative((Some(VerticalOrYSide::Vertical(v)), None)));
                    }
                },
                PositionOne::Center(center) => {
                    x_pos.push(PositionX::Center(center, None));
                    y_pos.push(PositionY::Center(center, None));
                }
                PositionOne::BlockAxis(block) => {
                    let resolved = resolve_block_axis(block, writing_mode);
                    y_pos.push(PositionY::Relative((Some(resolved), None)));
                }
                PositionOne::InlineAxis(inline) => {
                    let resolved = resolve_inline_axis(inline, writing_mode);
                    x_pos.push(PositionX::Relative((Some(resolved), None)));
                }
            },
            BgPosition::Two(two) => {
                match two {
                    PositionTwo::Axis(x, y) => {
                        x_pos.push(resolve_x_axis(x));
                        y_pos.push(resolve_y_axis(y));
                    }
                    PositionTwo::Relative(x_rel, y_rel) => {
                        match x_rel {
                            RelativeAxis::Center(center) => x_pos.push(PositionX::Center(center, None)),
                            RelativeAxis::Side(side) => x_pos
                                .push(PositionX::Relative((Some(resolve_horizontal_x_side(side, writing_mode)), None))),
                        }
                        match y_rel {
                            RelativeAxis::Center(center) => y_pos.push(PositionY::Center(center, None)),
                            RelativeAxis::Side(side) => y_pos
                                .push(PositionY::Relative((Some(resolve_vertical_y_side(side, writing_mode)), None))),
                        }
                    }
                    PositionTwo::AxisOrPercentage(x_pct, y_pct) => {
                        match x_pct {
                            XAxisOrLengthPercentage::XAxis(x_axis) => x_pos.push(resolve_x_axis(x_axis)),
                            XAxisOrLengthPercentage::LengthPercentage(lp) => {
                                x_pos.push(PositionX::Relative((None, Some(lp))));
                            }
                        }
                        match y_pct {
                            YAxisOrLengthPercentage::YAxis(y_axis) => y_pos.push(resolve_y_axis(y_axis)),
                            YAxisOrLengthPercentage::LengthPercentage(lp) => {
                                y_pos.push(PositionY::Relative((None, Some(lp))));
                            }
                        }
                    }
                    PositionTwo::BlockInline(block, inline) => {
                        let resolved_block = resolve_block_axis(block, writing_mode);
                        let resolved_inline = resolve_inline_axis(inline, writing_mode);
                        y_pos.push(PositionY::Relative((Some(resolved_block), None)));
                        x_pos.push(PositionX::Relative((Some(resolved_inline), None)));
                    }
                }
            }
            BgPosition::Three(three) => match three {
                PositionThree::RelativeHorizontal((horizontal, len_pct), rel_vertical_side) => {
                    let resolved_horizontal = resolve_horizontal_side(horizontal, writing_mode);
                    x_pos.push(PositionX::Relative((Some(resolved_horizontal), Some(len_pct))));
                    match rel_vertical_side {
                        RelativeVerticalSide::Center(center) => y_pos.push(PositionY::Center(center, None)),
                        RelativeVerticalSide::Vertical(vertical_side) => {
                            y_pos.push(PositionY::Relative((Some(VerticalOrYSide::Vertical(vertical_side)), None)));
                        }
                    }
                }
                PositionThree::RelativeVertical(rel_horizontal_side, (vertical, len_pct)) => {
                    let resolved_vertical = resolve_vertical_side(vertical, writing_mode);
                    y_pos.push(PositionY::Relative((Some(resolved_vertical), Some(len_pct))));
                    match rel_horizontal_side {
                        RelativeHorizontalSide::Center(center) => x_pos.push(PositionX::Center(center, None)),
                        RelativeHorizontalSide::Horizontal(horizontal_side) => {
                            x_pos.push(PositionX::Relative((
                                Some(HorizontalOrXSide::Horizontal(horizontal_side)),
                                None,
                            )));
                        }
                    }
                }
            },
            BgPosition::Four(four) => match four {
                PositionFour::BlockInline((block, x_len_pct), (inline, y_len_pct)) => {
                    let resolved_block = resolve_block_axis(block, writing_mode);
                    let resolved_inline = resolve_inline_axis(inline, writing_mode);
                    y_pos.push(PositionY::Relative((Some(resolved_block), Some(y_len_pct))));
                    x_pos.push(PositionX::Relative((Some(resolved_inline), Some(x_len_pct))));
                }
                PositionFour::StartEnd((x_side, x_len_pct), (y_side, y_len_pct)) => {
                    let resolved_x_side = resolve_horizontal_x_side(x_side, writing_mode);
                    let resolved_y_side = resolve_vertical_y_side(y_side, writing_mode);
                    x_pos.push(PositionX::Relative((Some(resolved_x_side), Some(x_len_pct))));
                    y_pos.push(PositionY::Relative((Some(resolved_y_side), Some(y_len_pct))));
                }
                PositionFour::XYPercentage((horizontal_side, x_len_pct), (vertical_side, y_len_pct)) => {
                    match horizontal_side {
                        HorizontalOrXSide::Horizontal(h) => {
                            x_pos.push(PositionX::Relative((Some(HorizontalOrXSide::Horizontal(h)), Some(x_len_pct))));
                        }
                        HorizontalOrXSide::XSide(xside) => {
                            x_pos.push(PositionX::Relative((Some(HorizontalOrXSide::XSide(xside)), Some(x_len_pct))));
                        }
                    }
                    match vertical_side {
                        VerticalOrYSide::Vertical(v) => {
                            y_pos.push(PositionY::Relative((Some(VerticalOrYSide::Vertical(v)), Some(y_len_pct))));
                        }
                        VerticalOrYSide::YSide(yside) => {
                            y_pos.push(PositionY::Relative((Some(VerticalOrYSide::YSide(yside)), Some(y_len_pct))));
                        }
                    }
                }
            },
        }
    }
}

impl CSSParsable for BackgroundPosition {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        let mut layers = Vec::new();
        let mut values = Vec::new();

        while let Some(cv) = stream.next_cv() {
            match cv {
                ComponentValue::Token(t) if t.kind == CssTokenKind::Comma => {
                    if values.is_empty() {
                        return Err(CssValueError::InvalidValue(
                            "Unexpected comma in background-position with no preceding values".into(),
                        ));
                    }
                    let mut value_stream = ComponentValueStream::from(&values);
                    let pos = BgPosition::parse(&mut value_stream)?;
                    layers.push(pos);
                    values.clear();
                }
                _ => values.push(cv.clone()),
            }
        }

        if !values.is_empty() {
            let mut value_stream = ComponentValueStream::from(&values);
            let pos = BgPosition::parse(&mut value_stream)?;
            layers.push(pos);
        } else if layers.is_empty() {
            return Err(CssValueError::InvalidValue("No valid BgPosition found for BackgroundPosition".into()));
        }

        Ok(Self(layers))
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut positions = Vec::new();
        let mut current_position = None;
        let mut horizontal_side = None;
        let mut length_percentage = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_position.is_some() {
                            return Err(CssValueError::InvalidValue(
                                "Multiple position keywords without a comma.".into(),
                            ));
                        } else if ident.eq_ignore_ascii_case("center") {
                            current_position = Some(PositionX::Center(Center::Center, length_percentage.take()));
                        } else if let Ok(h) = ident.parse() {
                            horizontal_side = Some(h);
                        } else {
                            return Err(CssValueError::InvalidValue(format!(
                                "Invalid horizontal side keyword: '{ident}'"
                            )));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        if length_percentage.is_some() {
                            return Err(CssValueError::InvalidValue("Duplicate length or percentage".into()));
                        }

                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                        length_percentage = Some(LengthPercentage::Length(Length::new(value.to_f64(), len_unit)));
                    }
                    CssTokenKind::Percentage(value) => {
                        if length_percentage.is_some() {
                            return Err(CssValueError::InvalidValue("Duplicate length or percentage".into()));
                        }
                        length_percentage = Some(LengthPercentage::Percentage(Percentage::new(value.to_f64())));
                    }
                    CssTokenKind::Comma => {
                        if let Some(pos) = current_position.take() {
                            if horizontal_side.is_some() || length_percentage.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Cannot have a center position with additional length/percentage when multiple positions are specified"
                                        .into())
                                );
                            }

                            positions.push(pos);
                        } else if horizontal_side.is_some() || length_percentage.is_some() {
                            positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(pos) = current_position.take() {
            if horizontal_side.is_some() || length_percentage.is_some() {
                return Err(CssValueError::InvalidValue(
                    "Cannot have a center position with additional length/percentage".into(),
                ));
            }

            positions.push(pos);
        } else if horizontal_side.is_some() || length_percentage.is_some() {
            positions.push(PositionX::Relative((horizontal_side.take(), length_percentage.take())));
        }

        if positions.is_empty() {
            Err(CssValueError::InvalidValue("No valid PositionX found for BackgroundPositionX.".into()))
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut positions = Vec::new();
        let mut current_position = None;
        let mut vertical_side = None;
        let mut length_percentage = None;

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_position.is_some() {
                            return Err(CssValueError::InvalidValue(
                                "Multiple position keywords without a comma.".into(),
                            ));
                        } else if ident.eq_ignore_ascii_case("center") {
                            current_position = Some(PositionY::Center(Center::Center, length_percentage.take()));
                        } else if let Ok(v) = ident.parse() {
                            vertical_side = Some(v);
                        } else {
                            return Err(CssValueError::InvalidValue(format!(
                                "Invalid vertical side keyword: '{ident}'"
                            )));
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        if length_percentage.is_some() {
                            return Err(CssValueError::InvalidValue("Duplicate length or percentage".into()));
                        }

                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;

                        length_percentage = Some(LengthPercentage::Length(Length::new(value.to_f64(), len_unit)));
                    }
                    CssTokenKind::Percentage(value) => {
                        if length_percentage.is_some() {
                            return Err(CssValueError::InvalidValue("Duplicate length or percentage".into()));
                        }
                        length_percentage = Some(LengthPercentage::Percentage(Percentage::new(value.to_f64())));
                    }
                    CssTokenKind::Comma => {
                        if let Some(pos) = current_position.take() {
                            if vertical_side.is_some() || length_percentage.is_some() {
                                return Err(CssValueError::InvalidValue(
                                    "Cannot have a center position with additional length/percentage when multiple positions are specified"
                                        .into())
                                );
                            }

                            positions.push(pos);
                        } else if vertical_side.is_some() || length_percentage.is_some() {
                            positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(pos) = current_position.take() {
            if vertical_side.is_some() || length_percentage.is_some() {
                return Err(CssValueError::InvalidValue(
                    "Cannot have a center position with additional length/percentage".into(),
                ));
            }

            positions.push(pos);
        } else if vertical_side.is_some() || length_percentage.is_some() {
            positions.push(PositionY::Relative((vertical_side.take(), length_percentage.take())));
        }

        if positions.is_empty() {
            Err(CssValueError::InvalidValue("No valid PositionY found for BackgroundPositionY.".into()))
        } else {
            Ok(Self(positions))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundRepeat(pub Vec<(RepeatStyle, RepeatStyle)>);

impl Default for BackgroundRepeat {
    fn default() -> Self {
        Self(vec![(RepeatStyle::Repeat, RepeatStyle::Repeat)])
    }
}

impl CSSParsable for BackgroundRepeat {
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut keywords = Vec::new();
        let mut current_pair = (None, None);

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_pair.0.is_some() && current_pair.1.is_some() {
                            return Err(CssValueError::InvalidValue(
                                "Too many repeat style keywords without a comma".into(),
                            ));
                        } else if ident.eq_ignore_ascii_case("repeat") {
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
                    _ => {}
                }
            }
        }

        if let (Some(first), Some(second)) = current_pair {
            keywords.push((first, second));
        } else if let (Some(first), None) = current_pair {
            keywords.push((first, first));
        }

        if keywords.is_empty() {
            Err(CssValueError::InvalidValue("No valid repeat style pairs found for BackgroundRepeat".into()))
        } else {
            Ok(Self(keywords))
        }
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
    fn parse(stream: &mut ComponentValueStream) -> Result<Self, CssValueError> {
        stream.skip_whitespace();
        let mut sizes = Vec::new();
        let mut current_size = None;
        let mut width_height_values = Vec::with_capacity(2);

        // TODO: calc();

        while let Some(cv) = stream.next_cv() {
            if let ComponentValue::Token(token) = cv {
                match &token.kind {
                    CssTokenKind::Ident(ident) => {
                        if current_size.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple size keywords without a comma".into()));
                        }

                        if ident.eq_ignore_ascii_case("auto") {
                            if width_height_values.len() > 2 {
                                return Err(CssValueError::InvalidValue("Too many width/height values".into()));
                            }

                            width_height_values.push(WidthHeightSize::Auto);
                        } else if ident.eq_ignore_ascii_case("cover") {
                            current_size = Some(Size::Cover);
                        } else if ident.eq_ignore_ascii_case("contain") {
                            current_size = Some(Size::Contain);
                        }
                    }
                    CssTokenKind::Dimension { value, unit } => {
                        if current_size.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple size keywords without a comma".into()));
                        } else if width_height_values.len() > 2 {
                            return Err(CssValueError::InvalidValue("Too many width/height values".into()));
                        }

                        let len_unit = unit
                            .parse::<LengthUnit>()
                            .map_err(|_| CssValueError::InvalidUnit(unit.clone()))?;
                        width_height_values.push(WidthHeightSize::Length(LengthPercentage::Length(Length::new(
                            value.to_f64(),
                            len_unit,
                        ))));
                    }
                    CssTokenKind::Percentage(value) => {
                        if current_size.is_some() {
                            return Err(CssValueError::InvalidValue("Multiple size keywords without a comma".into()));
                        } else if width_height_values.len() > 2 {
                            return Err(CssValueError::InvalidValue("Too many width/height values".into()));
                        }

                        width_height_values.push(WidthHeightSize::Length(LengthPercentage::Percentage(
                            Percentage::new(value.to_f64()),
                        )));
                    }
                    CssTokenKind::Comma => {
                        if let Some(size) = current_size.take() {
                            sizes.push(size);
                        } else if width_height_values.len() > 2 {
                            return Err(CssValueError::InvalidValue("Too many width/height values".into()));
                        } else if !width_height_values.is_empty() {
                            match width_height_values.len() {
                                1 if width_height_values[0] == WidthHeightSize::Auto => {
                                    sizes.push(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)));
                                }
                                1 => {
                                    let values = std::mem::take(&mut width_height_values);
                                    sizes.push(Size::WidthHeight(values[0].clone(), values.get(1).cloned()));
                                }
                                2 => {
                                    let values = std::mem::take(&mut width_height_values);
                                    sizes.push(Size::WidthHeight(values[0].clone(), Some(values[1].clone())));
                                }
                                _ => return Err(CssValueError::InvalidValue("Too many width/height values".into())),
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if let Some(size) = current_size.take() {
            sizes.push(size);
        } else if width_height_values.len() > 2 {
            return Err(CssValueError::InvalidValue("Too many width/height values".into()));
        } else if !width_height_values.is_empty() {
            match width_height_values.len() {
                1 if width_height_values[0] == WidthHeightSize::Auto => {
                    sizes.push(Size::WidthHeight(WidthHeightSize::Auto, Some(WidthHeightSize::Auto)));
                }
                1 => {
                    let values = std::mem::take(&mut width_height_values);
                    sizes.push(Size::WidthHeight(values[0].clone(), values.get(1).cloned()));
                }
                2 => {
                    let values = std::mem::take(&mut width_height_values);
                    sizes.push(Size::WidthHeight(values[0].clone(), Some(values[1].clone())));
                }
                _ => return Err(CssValueError::InvalidValue("Too many width/height values".into())),
            }
        }

        if sizes.is_empty() {
            Err(CssValueError::InvalidValue("No valid Size found for BackgroundSize".into()))
        } else {
            Ok(Self(sizes))
        }
    }
}

#[cfg(test)]
mod tests {
    use css_cssom::CssToken;
    use css_values::image::Image;

    use super::*;

    #[test]
    fn test_background_attachment() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("scroll".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("fixed".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("local".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundAttachment::parse(&mut stream);
        assert!(result.is_ok());
        let background_attachment = result.unwrap();
        assert_eq!(background_attachment.0.len(), 3);
        assert_eq!(background_attachment.0[0], Attachment::Scroll);
        assert_eq!(background_attachment.0[1], Attachment::Fixed);
        assert_eq!(background_attachment.0[2], Attachment::Local);
    }

    #[test]
    fn test_background_attachment_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("fixed".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundAttachment::parse(&mut stream);
        assert!(result.is_ok());
        let background_attachment = result.unwrap();
        assert_eq!(background_attachment.0.len(), 1);
        assert_eq!(background_attachment.0[0], Attachment::Fixed);
    }

    #[test]
    fn test_background_attachment_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("scroll".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("fixed".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("local".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundAttachment::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_attachment_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundAttachment::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_blend_mode() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("normal".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("darken".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("color".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundBlendMode::parse(&mut stream);

        assert!(result.is_ok());
        let background_blend_mode = result.unwrap();
        assert_eq!(background_blend_mode.0.len(), 3);
        assert_eq!(background_blend_mode.0[0], BlendMode::Normal);
        assert_eq!(background_blend_mode.0[1], BlendMode::Darken);
        assert_eq!(background_blend_mode.0[2], BlendMode::Color);
    }

    #[test]
    fn test_background_blend_mode_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("multiply".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundBlendMode::parse(&mut stream);

        assert!(result.is_ok());
        let background_blend_mode = result.unwrap();
        assert_eq!(background_blend_mode.0.len(), 1);
        assert_eq!(background_blend_mode.0[0], BlendMode::Multiply);
    }

    #[test]
    fn test_background_blend_mode_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("normal".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("darken".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("color".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundBlendMode::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_blend_mode_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundBlendMode::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_clip() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("border-box".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("text".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("border-area".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundClip::parse(&mut stream);

        assert!(result.is_ok());
        let background_clip = result.unwrap();
        assert_eq!(background_clip.0.len(), 2);
        assert_eq!(background_clip.0[0], BgClip::Visual(VisualBox::Border));
        assert_eq!(background_clip.0[1], BgClip::Clip(Clip::Text, Some(Clip::BorderArea)));
    }

    #[test]
    fn test_background_clip_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("padding-box".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundClip::parse(&mut stream);

        assert!(result.is_ok());
        let background_clip = result.unwrap();
        assert_eq!(background_clip.0.len(), 1);
        assert_eq!(background_clip.0[0], BgClip::Visual(VisualBox::Padding));
    }

    #[test]
    fn test_background_clip_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("border-box".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("text".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("border-area".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundClip::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_clip_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundClip::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_image() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("none".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Url("https://example.com/image.png".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundImage::parse(&mut stream);

        assert!(result.is_ok());
        let background_image = result.unwrap();
        assert_eq!(background_image.0.len(), 2);
        assert_eq!(background_image.0[0], Image::None);
        assert_eq!(background_image.0[1], Image::Url("https://example.com/image.png".to_string()));
    }

    #[test]
    fn test_background_image_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Url("https://example.com/image.png".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundImage::parse(&mut stream);

        assert!(result.is_ok());
        let background_image = result.unwrap();
        assert_eq!(background_image.0.len(), 1);
        assert_eq!(background_image.0[0], Image::Url("https://example.com/image.png".to_string()));
    }

    #[test]
    fn test_background_image_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("none".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Url("https://example.com/image.png".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundImage::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_image_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundImage::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_origin() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("content-box".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("padding-box".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundOrigin::parse(&mut stream);

        assert!(result.is_ok());
        let background_origin = result.unwrap();
        assert_eq!(background_origin.0.len(), 2);
        assert_eq!(background_origin.0[0], VisualBox::Content);
        assert_eq!(background_origin.0[1], VisualBox::Padding);
    }

    #[test]
    fn test_background_origin_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("border-box".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundOrigin::parse(&mut stream);

        assert!(result.is_ok());
        let background_origin = result.unwrap();
        assert_eq!(background_origin.0.len(), 1);
        assert_eq!(background_origin.0[0], VisualBox::Border);
    }

    #[test]
    fn test_background_origin_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("content-box".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("padding-box".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundOrigin::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_origin_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundOrigin::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_x() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionX::parse(&mut stream);

        assert!(result.is_ok());
        let background_position_x = result.unwrap();
        assert_eq!(background_position_x.0.len(), 2);
        assert_eq!(
            background_position_x.0[0],
            PositionX::Relative((Some(HorizontalOrXSide::Horizontal(HorizontalSide::Left)), None))
        );
        assert_eq!(background_position_x.0[1], PositionX::Center(Center::Center, None));
    }

    #[test]
    fn test_background_position_x_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("right".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionX::parse(&mut stream);

        assert!(result.is_ok());
        let background_position_x = result.unwrap();
        assert_eq!(background_position_x.0.len(), 1);
        assert_eq!(
            background_position_x.0[0],
            PositionX::Relative((Some(HorizontalOrXSide::Horizontal(HorizontalSide::Right)), None))
        );
    }

    #[test]
    fn test_background_position_x_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionX::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_x_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionX::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_x_invalid_center() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: 50.into(),
                    unit: "px".to_string(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionX::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_y() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionY::parse(&mut stream);

        assert!(result.is_ok());
        let background_position_y = result.unwrap();
        assert_eq!(background_position_y.0.len(), 2);
        assert_eq!(
            background_position_y.0[0],
            PositionY::Relative((Some(VerticalOrYSide::Vertical(VerticalSide::Top)), None))
        );
        assert_eq!(background_position_y.0[1], PositionY::Center(Center::Center, None));
    }

    #[test]
    fn test_background_position_y_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("bottom".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionY::parse(&mut stream);

        assert!(result.is_ok());
        let background_position_y = result.unwrap();
        assert_eq!(background_position_y.0.len(), 1);
        assert_eq!(
            background_position_y.0[0],
            PositionY::Relative((Some(VerticalOrYSide::Vertical(VerticalSide::Bottom)), None))
        );
    }

    #[test]
    fn test_background_position_y_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionY::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_y_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionY::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_position_y_invalid_center() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: 50.into(),
                    unit: "px".to_string(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPositionY::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_repeat() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("repeat".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("space".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundRepeat::parse(&mut stream);

        assert!(result.is_ok());
        let background_repeat = result.unwrap();
        assert_eq!(background_repeat.0.len(), 2);
        assert_eq!(background_repeat.0[0], (RepeatStyle::Repeat, RepeatStyle::Repeat));
        assert_eq!(background_repeat.0[1], (RepeatStyle::Space, RepeatStyle::Space));
    }

    #[test]
    fn test_background_repeat_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("no-repeat".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundRepeat::parse(&mut stream);

        assert!(result.is_ok());
        let background_repeat = result.unwrap();
        assert_eq!(background_repeat.0.len(), 1);
        assert_eq!(background_repeat.0[0], (RepeatStyle::NoRepeat, RepeatStyle::NoRepeat));
    }

    #[test]
    fn test_background_repeat_shorthand() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("repeat-x".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundRepeat::parse(&mut stream);

        assert!(result.is_ok());
        let background_repeat = result.unwrap();
        assert_eq!(background_repeat.0.len(), 1);
        assert_eq!(background_repeat.0[0], (RepeatStyle::Repeat, RepeatStyle::NoRepeat));
    }

    #[test]
    fn test_background_repeat_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("repeat".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("space".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("repeat".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundRepeat::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_repeat_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundRepeat::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_size() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("cover".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: 100.into(),
                    unit: "px".to_string(),
                },
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: 200.into(),
                    unit: "px".to_string(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundSize::parse(&mut stream);

        assert!(result.is_ok());
        let background_size = result.unwrap();
        assert_eq!(background_size.0.len(), 2);
        assert_eq!(background_size.0[0], Size::Cover);
        assert_eq!(
            background_size.0[1],
            Size::WidthHeight(
                WidthHeightSize::Length(LengthPercentage::Length(Length::new(100.0, LengthUnit::Px))),
                Some(WidthHeightSize::Length(LengthPercentage::Length(Length::new(200.0, LengthUnit::Px))))
            )
        );
    }

    #[test]
    fn test_background_size_single() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("contain".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundSize::parse(&mut stream);

        assert!(result.is_ok());
        let background_size = result.unwrap();
        assert_eq!(background_size.0.len(), 1);
        assert_eq!(background_size.0[0], Size::Contain);
    }

    #[test]
    fn test_background_size_invalid_whitespace() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("cover".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("cover".to_string()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundSize::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_size_invalid_mix() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("cover".to_string()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Dimension {
                    value: 100.into(),
                    unit: "px".to_string(),
                },
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundSize::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_background_size_invalid() {
        let input = vec![ComponentValue::Token(CssToken {
            kind: CssTokenKind::Ident("invalid".to_string()),
            position: None,
        })];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundSize::parse(&mut stream);

        assert!(result.is_err());
    }

    #[test]
    fn test_bg_position_multiple_layers() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("left".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("top".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Comma,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("right".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("bottom".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPosition::parse(&mut stream);

        assert!(result.is_ok());
        let bg_pos = result.unwrap();
        assert_eq!(bg_pos.0.len(), 2);
        assert_eq!(
            bg_pos.0[0],
            BgPosition::Two(PositionTwo::Axis(
                XAxis::Horizontal(HorizontalSide::Left),
                YAxis::Vertical(VerticalSide::Top)
            ))
        );
        assert_eq!(
            bg_pos.0[1],
            BgPosition::Two(PositionTwo::Axis(
                XAxis::Horizontal(HorizontalSide::Right),
                YAxis::Vertical(VerticalSide::Bottom)
            ))
        );
    }

    #[test]
    fn test_bg_position_single_layer() {
        let input = vec![
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Whitespace,
                position: None,
            }),
            ComponentValue::Token(CssToken {
                kind: CssTokenKind::Ident("center".into()),
                position: None,
            }),
        ];

        let mut stream = ComponentValueStream::from(&input);
        let result = BackgroundPosition::parse(&mut stream);

        assert!(result.is_ok());
        let bg_pos = result.unwrap();
        assert_eq!(bg_pos.0.len(), 1);
        assert_eq!(
            bg_pos.0[0],
            BgPosition::Two(PositionTwo::Axis(XAxis::Center(Center::Center), YAxis::Center(Center::Center)))
        );
    }
}
