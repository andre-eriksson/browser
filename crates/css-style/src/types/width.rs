use crate::{
    types::{Parseable, global::Global, length::Length},
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Width {
    Percentage(f32),
    Length(Length),
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
    Global(Global),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MaxWidth {
    Length(Length),
    Percentage(f32),
    None,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
    Global(Global),
}

impl Parseable for Width {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(Self::Global(global));
        }

        if value.contains('%')
            && let Some(percentage) = Unit::resolve_percentage(value)
        {
            return Some(Self::Percentage(percentage));
        }

        if let Some(length) = Length::parse(value) {
            return Some(Self::Length(length));
        }

        if value.eq_ignore_ascii_case("auto") {
            return Some(Self::Auto);
        } else if value.eq_ignore_ascii_case("max-content") {
            return Some(Self::MaxContent);
        } else if value.eq_ignore_ascii_case("min-content") {
            return Some(Self::MinContent);
        } else if value.eq_ignore_ascii_case("fit-content") {
            return Some(Self::FitContent(None)); // TODO: Fix?
        } else if value.eq_ignore_ascii_case("stretch") {
            return Some(Self::Stretch);
        }

        None
    }
}

impl Parseable for MaxWidth {
    fn parse(value: &str) -> Option<Self> {
        if let Some(global) = Global::parse(value) {
            return Some(Self::Global(global));
        }

        if value.contains('%')
            && let Some(percentage) = Unit::resolve_percentage(value)
        {
            return Some(Self::Percentage(percentage));
        }

        if let Some(length) = Length::parse(value) {
            return Some(Self::Length(length));
        }

        if value.eq_ignore_ascii_case("none") {
            return Some(Self::None);
        } else if value.eq_ignore_ascii_case("max-content") {
            return Some(Self::MaxContent);
        } else if value.eq_ignore_ascii_case("min-content") {
            return Some(Self::MinContent);
        } else if value.eq_ignore_ascii_case("fit-content") {
            return Some(Self::FitContent(None)); // TODO: Fix?
        } else if value.eq_ignore_ascii_case("stretch") {
            return Some(Self::Stretch);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_width() {
        assert_eq!(Width::parse("50%"), Some(Width::Percentage(50.0)));
        assert_eq!(Width::parse("auto"), Some(Width::Auto));
        assert_eq!(Width::parse("max-content"), Some(Width::MaxContent));
        assert_eq!(Width::parse("min-content"), Some(Width::MinContent));
        assert_eq!(Width::parse("fit-content"), Some(Width::FitContent(None)));
        assert_eq!(Width::parse("stretch"), Some(Width::Stretch));
    }
}
