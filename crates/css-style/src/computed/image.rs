use css_values::image::{Gradient, Image};
use url::Url;

use crate::{AbsoluteContext, properties::background::BackgroundImage};

#[derive(Debug, Clone, PartialEq)]
pub enum ComputedImage {
    None,
    Url(Url),
    Gradient(Gradient),
}

impl From<ComputedImage> for Image {
    fn from(computed: ComputedImage) -> Self {
        match computed {
            ComputedImage::None => Self::None,
            ComputedImage::Url(url) => Self::Url(url.to_string()),
            ComputedImage::Gradient(gradient) => Self::Gradient(Box::new(gradient)),
        }
    }
}

impl ComputedImage {
    pub fn resolve(image: Image, absolute_ctx: &AbsoluteContext) -> Result<Self, String> {
        match image {
            Image::Url(url) => Ok(Self::Url(
                absolute_ctx
                    .document_url
                    .join(&url)
                    .map_err(|e| format!("Failed to resolve URL: {e}"))?,
            )),
            Image::Gradient(gradient) => Ok(Self::Gradient(*gradient)),
            Image::None => Ok(Self::None),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ComputedBackgroundImage(pub Vec<ComputedImage>);

impl From<ComputedBackgroundImage> for BackgroundImage {
    fn from(computed: ComputedBackgroundImage) -> Self {
        Self(computed.0.into_iter().map(ComputedImage::into).collect())
    }
}

impl ComputedBackgroundImage {
    pub const fn none() -> Self {
        Self(vec![])
    }

    pub fn resolve(images: Vec<Image>, absolute_ctx: &AbsoluteContext) -> Result<Self, String> {
        images
            .into_iter()
            .map(|image| ComputedImage::resolve(image, absolute_ctx))
            .collect::<Result<Vec<_>, _>>()
            .map(ComputedBackgroundImage)
    }
}
