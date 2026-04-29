use std::str::FromStr;

use css_cssom::{ComponentValue, ComponentValueStream, CssTokenKind};

use crate::{
    CSSParsable,
    color::Color,
    combination::LengthPercentage,
    image::Image,
    numeric::Percentage,
    quantity::{Angle, Length, Resolution, Time},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxComponent {
    Length,
    Number,
    Percentage,
    LengthPercentage,
    Color,
    Image,
    Url,
    Integer,
    Angle,
    Time,
    Resolution,
    TransformFunction,
    TransformList,
    CustomIdent,
    // For literal idents like "left | right"
    Ident(String),
}

impl FromStr for SyntaxComponent {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("length") {
            Ok(Self::Length)
        } else if s.eq_ignore_ascii_case("number") {
            Ok(Self::Number)
        } else if s.eq_ignore_ascii_case("percentage") {
            Ok(Self::Percentage)
        } else if s.eq_ignore_ascii_case("length-percentage") {
            Ok(Self::LengthPercentage)
        } else if s.eq_ignore_ascii_case("color") {
            Ok(Self::Color)
        } else if s.eq_ignore_ascii_case("image") {
            Ok(Self::Image)
        } else if s.eq_ignore_ascii_case("url") {
            Ok(Self::Url)
        } else if s.eq_ignore_ascii_case("integer") {
            Ok(Self::Integer)
        } else if s.eq_ignore_ascii_case("angle") {
            Ok(Self::Angle)
        } else if s.eq_ignore_ascii_case("time") {
            Ok(Self::Time)
        } else if s.eq_ignore_ascii_case("resolution") {
            Ok(Self::Resolution)
        } else if s.eq_ignore_ascii_case("transform-function") {
            Ok(Self::TransformFunction)
        } else if s.eq_ignore_ascii_case("transform-list") {
            Ok(Self::TransformList)
        } else if s.starts_with("--") {
            Ok(Self::CustomIdent) // Custom properties
        } else if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Ok(Self::Ident(s.to_string()))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertySyntax {
    /// The "*" universal syntax - any value is valid
    Universal,
    /// Specific type(s) with optional multipliers
    Typed(Vec<SyntaxComponent>),
}

#[derive(Debug, Clone)]
pub struct PropertyDescriptor {
    pub name: String,
    pub syntax: PropertySyntax,
    pub inherits: bool,
    pub initial_value: Option<Vec<ComponentValue>>,
}

impl PropertySyntax {
    #[must_use]
    pub fn validate(&self, values: &[ComponentValue]) -> bool {
        match self {
            Self::Universal => true,
            Self::Typed(components) => {
                let mut stream = ComponentValueStream::new(values);

                components.iter().any(|comp| match comp {
                    SyntaxComponent::Angle => Angle::parse(&mut stream).is_ok(),
                    SyntaxComponent::Color => Color::parse(&mut stream).is_ok(),
                    SyntaxComponent::CustomIdent => {
                        stream.next_non_whitespace().is_some_and(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Ident(_))))
                    }
                    SyntaxComponent::Ident(ident) => {
                        stream.next_non_whitespace().is_some_and(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Ident(idt) if idt.eq_ignore_ascii_case(ident))))
                    }
                    SyntaxComponent::Image => Image::parse(&mut stream).is_ok(),
                    SyntaxComponent::Url => {
                        stream.next_non_whitespace().is_some_and(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Url(_))))
                    }
                    SyntaxComponent::Integer => {
                        stream.next_non_whitespace().is_some_and(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Number(n) if n.is_integer())))
                    }
                    SyntaxComponent::Length => Length::parse(&mut stream).is_ok(),
                    SyntaxComponent::LengthPercentage => LengthPercentage::parse(&mut stream).is_ok(),
                    SyntaxComponent::Number => {
                        stream.next_non_whitespace().is_some_and(|cv| matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Number(_))))
                    }
                    SyntaxComponent::Percentage => Percentage::parse(&mut stream).is_ok(),
                    SyntaxComponent::Resolution => Resolution::parse(&mut stream).is_ok(),
                    SyntaxComponent::Time => Time::parse(&mut stream).is_ok(),
                    SyntaxComponent::TransformFunction => false, // TODO: Support transform functions
                    SyntaxComponent::TransformList => false,     // TODO: Support transform functions
                })
            }
        }
    }
}
