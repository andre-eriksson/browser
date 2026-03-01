use css_cssom::{ComponentValue, CssTokenKind, Function};

use crate::properties::gradient::Gradient;

#[derive(Debug, Clone, PartialEq)]
pub enum Image {
    Url(String),
    Gradient(Gradient),
    // TODO: Element()
    // TODO: Image()
    // TODO: CrossFade()
    // TODO: ImageSet()
    // TODO: Paint()
}

impl TryFrom<&Function> for Image {
    type Error = String;

    fn try_from(value: &Function) -> Result<Self, Self::Error> {
        if let Ok(gradient) = Gradient::try_from(value) {
            Ok(Image::Gradient(gradient))
        } else if value.name.eq_ignore_ascii_case("url") {
            if let Some(ComponentValue::Token(token)) = value.value.first() {
                if let CssTokenKind::String(s) = &token.kind {
                    Ok(Image::Url(s.clone()))
                } else {
                    Err("Expected a string token in url() function".to_string())
                }
            } else {
                Err("Expected at least one argument in url() function".to_string())
            }
        } else {
            Err(format!("Unknown image function: '{}'", value.name))
        }
    }
}
