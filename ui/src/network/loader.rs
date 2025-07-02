use api::sender::NetworkMessage;
use egui::{
    ColorImage, generate_loader_id,
    load::{ImageLoadResult, ImageLoader, ImagePoll},
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error};

type ImageCache = Arc<Mutex<HashMap<String, Arc<ColorImage>>>>;

/// A loader for images that fetches them over the network using a Tokio channel.
///
/// # Fields
/// * `network_sender` - An unbounded sender for sending network messages to the backend.
/// * `cache` - A thread-safe cache for storing images, allowing for quick retrieval without repeated network requests.
#[derive(Debug, Clone)]
pub struct NetworkLoader {
    pub network_sender: mpsc::UnboundedSender<NetworkMessage>,
    pub cache: ImageCache,
}

/// Loads an image from raw bytes and converts it to a `ColorImage`.
fn load_image_from_bytes(bytes: &[u8]) -> Result<ColorImage, String> {
    let image = image::load_from_memory(bytes)
        .map_err(|e| format!("Failed to load image from bytes: {:?}", e))?;

    let rgba_image = image.to_rgba8();
    let size = [rgba_image.width() as usize, rgba_image.height() as usize];
    let pixels = rgba_image
        .pixels()
        .map(|p| egui::Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
        .collect();

    Ok(ColorImage { size, pixels })
}

impl ImageLoader for NetworkLoader {
    fn id(&self) -> &str {
        return generate_loader_id!(NetworkLoader);
    }

    fn load(
        &self,
        ctx: &egui::Context,
        uri: &str,
        _size_hint: egui::SizeHint,
    ) -> egui::load::ImageLoadResult {
        let mut cache = self.cache.lock().unwrap();

        if cache.get(&format!("error:{}", uri)).is_some() {
            return ImageLoadResult::Err(egui::load::LoadError::NotSupported);
        }

        if let Some(cached_image) = cache.get(uri) {
            debug!("[NetworkLoader] ✅ FOUND cached image for URI: {}", uri);
            return ImageLoadResult::Ok(ImagePoll::Ready {
                image: cached_image.clone(),
            });
        }

        if cache.contains_key(&format!("pending:{}", uri)) {
            debug!("[NetworkLoader] Image still loading for URI: {}", uri);
            return ImageLoadResult::Ok(ImagePoll::Pending { size: None });
        }

        cache.insert(format!("pending:{}", uri), Arc::new(ColorImage::example()));

        let uri_str = uri.to_string();
        let ctx_clone = ctx.clone();
        let cache_clone = Arc::clone(&self.cache);
        let network_sender = self.network_sender.clone();

        tokio::spawn(async move {
            let (response_tx, response_rx) = oneshot::channel();

            let _ = network_sender.send(NetworkMessage::FetchContent {
                url: uri_str.to_string(),
                headers: None,
                method: None,
                body: None,
                tag_name: "img".to_string(),
                response: response_tx,
            });

            match response_rx.await {
                Ok(result) => match result {
                    Ok(response) => {
                        if !response.status().is_success() {
                            debug!(
                                "NetworkLoader: Failed to fetch image for URI {}: {}",
                                uri_str,
                                response.status()
                            );
                            let mut cache = cache_clone.lock().unwrap();
                            cache.remove(&format!("pending:{}", uri_str));
                            cache.insert(
                                format!("error:{}", uri_str),
                                Arc::new(ColorImage::example()),
                            );
                            return;
                        }

                        match response.bytes().await {
                            Ok(bytes) => match load_image_from_bytes(&bytes) {
                                Ok(image) => {
                                    let mut cache = cache_clone.lock().unwrap();
                                    cache.remove(&format!("pending:{}", uri_str));

                                    debug!(
                                        "[NetworkLoader] 💾 Storing image with key: '{}'",
                                        uri_str
                                    );
                                    cache.insert(uri_str.clone(), Arc::new(image));

                                    ctx_clone.request_repaint();
                                }
                                Err(err) => {
                                    debug!(
                                        "[NetworkLoader] Failed to decode image for URI {}: {}",
                                        uri_str, err
                                    );
                                    let mut cache = cache_clone.lock().unwrap();
                                    cache.remove(&format!("pending:{}", uri_str));
                                    cache.insert(
                                        format!("error:{}", uri_str),
                                        Arc::new(ColorImage::example()),
                                    );
                                    ctx_clone.request_repaint();
                                }
                            },
                            Err(err) => {
                                debug!(
                                    "[NetworkLoader] Failed to read bytes for URI {}: {}",
                                    uri_str, err
                                );
                                let mut cache = cache_clone.lock().unwrap();
                                cache.remove(&format!("pending:{}", uri_str));
                                cache.insert(
                                    format!("error:{}", uri_str),
                                    Arc::new(ColorImage::example()),
                                );
                                ctx_clone.request_repaint();
                            }
                        }
                    }
                    Err(err) => {
                        debug!(
                            "[NetworkLoader] Failed to fetch image for URI {}: {}, removing from cache",
                            uri_str, err
                        );
                        let mut cache = cache_clone.lock().unwrap();
                        cache.remove(&format!("pending:{}", uri_str));
                        cache.insert(
                            format!("error:{}", uri_str),
                            Arc::new(ColorImage::example()),
                        );
                        ctx_clone.request_repaint();
                    }
                },
                Err(err) => {
                    error!(
                        "[NetworkLoader] Failed to receive response for URI {}: {}",
                        uri_str, err
                    );
                    let mut cache = cache_clone.lock().unwrap();
                    cache.remove(&format!("pending:{}", uri_str));
                    cache.insert(
                        format!("error:{}", uri_str),
                        Arc::new(ColorImage::example()),
                    );
                    ctx_clone.request_repaint();
                }
            }
        });

        ImageLoadResult::Ok(ImagePoll::Pending { size: None })
    }

    fn forget(&self, uri: &str) {
        let mut cache = self.cache.lock().unwrap();
        cache.remove(uri);
        cache.remove(&format!("pending:{}", uri));
        cache.remove(&format!("error:{}", uri));
        debug!("[NetworkLoader] Forgot image for URI: {}", uri);
    }

    fn forget_all(&self) {
        let mut cache = self.cache.lock().unwrap();
        let count = cache.len();
        cache.clear();
        debug!("[NetworkLoader] Cleared {} cached images", count);
    }

    fn byte_size(&self) -> usize {
        let cache = self.cache.lock().unwrap();
        cache
            .values()
            .map(|img| img.pixels.len() * std::mem::size_of::<egui::Color32>())
            .sum()
    }
}
