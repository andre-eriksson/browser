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

#[derive(Debug, Clone, PartialEq)]
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
            Ok(SyntaxComponent::Length)
        } else if s.eq_ignore_ascii_case("number") {
            Ok(SyntaxComponent::Number)
        } else if s.eq_ignore_ascii_case("percentage") {
            Ok(SyntaxComponent::Percentage)
        } else if s.eq_ignore_ascii_case("length-percentage") {
            Ok(SyntaxComponent::LengthPercentage)
        } else if s.eq_ignore_ascii_case("color") {
            Ok(SyntaxComponent::Color)
        } else if s.eq_ignore_ascii_case("image") {
            Ok(SyntaxComponent::Image)
        } else if s.eq_ignore_ascii_case("url") {
            Ok(SyntaxComponent::Url)
        } else if s.eq_ignore_ascii_case("integer") {
            Ok(SyntaxComponent::Integer)
        } else if s.eq_ignore_ascii_case("angle") {
            Ok(SyntaxComponent::Angle)
        } else if s.eq_ignore_ascii_case("time") {
            Ok(SyntaxComponent::Time)
        } else if s.eq_ignore_ascii_case("resolution") {
            Ok(SyntaxComponent::Resolution)
        } else if s.eq_ignore_ascii_case("transform-function") {
            Ok(SyntaxComponent::TransformFunction)
        } else if s.eq_ignore_ascii_case("transform-list") {
            Ok(SyntaxComponent::TransformList)
        } else if s.starts_with("--") {
            Ok(SyntaxComponent::CustomIdent) // Custom properties
        } else if s.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
            Ok(SyntaxComponent::Ident(s.to_string()))
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
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
    pub fn validate(&self, values: &[ComponentValue]) -> bool {
        match self {
            PropertySyntax::Universal => true,
            PropertySyntax::Typed(components) => {
                let stream = ComponentValueStream::new(values);

                components.iter().any(|comp| match comp {
                    SyntaxComponent::Angle => Angle::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::Color => Color::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::CustomIdent => {
                        if let Some(cv) = stream.clone().next_non_whitespace() {
                            matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Ident(_)))
                        } else {
                            false
                        }
                    }
                    SyntaxComponent::Ident(ident) => {
                        if let Some(cv) = stream.clone().next_non_whitespace() {
                            matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Ident(idt) if idt.eq_ignore_ascii_case(ident)))
                        } else {
                            false
                        }
                    }
                    SyntaxComponent::Image => Image::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::Url => {
                        if let Some(cv) = stream.clone().next_non_whitespace() {
                            matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Url(_)))
                        } else {
                            false
                        }
                    }
                    SyntaxComponent::Integer => {
                        if let Some(cv) = stream.clone().next_non_whitespace() {
                            matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Number(n) if n.is_integer()))
                        } else {
                            false
                        }
                    }
                    SyntaxComponent::Length => Length::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::LengthPercentage => LengthPercentage::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::Number => {
                        if let Some(cv) = stream.clone().next_non_whitespace() {
                            matches!(cv, ComponentValue::Token(token) if matches!(&token.kind, CssTokenKind::Number(_)))
                        } else {
                            false
                        }
                    }
                    SyntaxComponent::Percentage => Percentage::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::Resolution => Resolution::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::Time => Time::parse(&mut stream.clone()).is_ok(),
                    SyntaxComponent::TransformFunction => false, // TODO: Support transform functions
                    SyntaxComponent::TransformList => false,     // TODO: Support transform functions
                })
            }
        }
    }
}
