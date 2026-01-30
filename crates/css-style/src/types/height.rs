use crate::{
    types::{Parseable, global::Global, length::Length},
    unit::Unit,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Height {
    Percentage(f32),
    Length(Length),
    Auto,
    MaxContent,
    MinContent,
    FitContent(Option<Length>),
    Stretch,
    Global(Global),
}

impl Parseable for Height {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_height() {
        assert_eq!(Height::parse("50%"), Some(Height::Percentage(50.0)));
        assert_eq!(Height::parse("auto"), Some(Height::Auto));
        assert_eq!(Height::parse("max-content"), Some(Height::MaxContent));
        assert_eq!(Height::parse("min-content"), Some(Height::MinContent));
        assert_eq!(Height::parse("fit-content"), Some(Height::FitContent(None)));
        assert_eq!(Height::parse("stretch"), Some(Height::Stretch));
        assert_eq!(
            Height::parse("inherit"),
            Some(Height::Global(Global::Inherit))
        );
        assert_eq!(
            Height::parse("100px"),
            Some(Height::Length(Length::parse("100px").unwrap()))
        );
        assert_eq!(Height::parse("unknown"), None);
    }
}
