use url::Url;

use crate::{
    AbsoluteContext,
    image::Image,
    properties::{background::BackgroundImage, gradient::Gradient},
};

#[derive(Debug, Clone, PartialEq)]
pub enum ComputedImage {
    Url(Url),
    Gradient(Gradient),
}

impl From<ComputedImage> for Image {
    fn from(computed: ComputedImage) -> Self {
        match computed {
            ComputedImage::Url(url) => Image::Url(url.to_string()),
            ComputedImage::Gradient(gradient) => Image::Gradient(gradient),
        }
    }
}

impl ComputedImage {
    pub fn resolve(image: Image, absolute_ctx: &AbsoluteContext) -> Result<Self, String> {
        match image {
            Image::Url(url) => Ok(ComputedImage::Url(match absolute_ctx.document_url {
                Some(base) => base
                    .join(&url)
                    .map_err(|e| format!("Failed to resolve URL: {}", e))?,
                None => Url::parse(&url).map_err(|e| format!("Failed to parse URL: {}", e))?,
            })),
            Image::Gradient(gradient) => Ok(ComputedImage::Gradient(gradient)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ComputedBackgroundImage(pub Vec<ComputedImage>);

impl From<ComputedBackgroundImage> for BackgroundImage {
    fn from(computed: ComputedBackgroundImage) -> Self {
        BackgroundImage(computed.0.into_iter().map(ComputedImage::into).collect())
    }
}

impl ComputedBackgroundImage {
    pub fn none() -> Self {
        ComputedBackgroundImage(vec![])
    }

    pub fn resolve(images: Vec<Image>, absolute_ctx: &AbsoluteContext) -> Result<Self, String> {
        let mut computed_images = Vec::new();
        for image in images {
            computed_images.push(ComputedImage::resolve(image, absolute_ctx)?);
        }
        Ok(ComputedBackgroundImage(computed_images))
    }
}
