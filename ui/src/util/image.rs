use iced::window::{Icon, icon::from_rgba};

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
