use iced::window::{Icon, icon::from_rgba};
use image::GenericImageView;
use renderer::DecodedImageData;

/// Loads an icon from a byte vector and converts it to an Iced window icon.
///
/// # Arguments
/// * `data` - A vector of bytes representing the icon image data.
///
/// # Returns
/// An `Icon` that can be used in an Iced application.
pub fn load_icon(data: Vec<u8>) -> Icon {
    let image = image::load_from_memory(&data)
        .expect("Failed to load icon image")
        .into_rgba8();

    let (width, height) = image.dimensions();
    let window_icon = from_rgba(image.into_raw(), width, height);

    if let Err(ref e) = window_icon {
        eprintln!("Failed to create window icon: {}", e);
    }
    window_icon.unwrap()
}

/// Decode raw image bytes into RGBA pixel data.
pub fn decode_image_bytes(url: String, bytes: &[u8]) -> Result<DecodedImageData, String> {
    let img = image::load_from_memory(bytes).map_err(|e| format!("Failed to decode image {}: {}", url, e))?;

    let (width, height) = img.dimensions();
    let rgba = img.to_rgba8().into_raw();

    Ok(DecodedImageData {
        rgba,
        width,
        height,
    })
}
