use std::collections::HashMap;

/// Metadata for a decoded image: its intrinsic CSS-pixel dimensions and the
/// pre-resolved Vary string used for exact disk-cache lookups at render time.
#[derive(Debug, Clone)]
pub struct ImageMeta {
    /// Intrinsic width in CSS pixels.
    pub width: f32,
    /// Intrinsic height in CSS pixels.
    pub height: f32,
    /// Pre-resolved Vary string computed from the HTTP response headers at
    /// fetch time.  May be empty if the response had no Vary header.
    pub vary_key: String,
}

/// Carries known image metadata (from previously decoded images) into the
/// layout pipeline so that images whose intrinsic sizes are already available
/// can be laid out with their real dimensions instead of placeholders.
///
/// This is the key piece of the relayout system: after an image is fetched and
/// decoded, its [`ImageMeta`] is stored here and a full relayout is performed.
/// The inline layout pass checks this context when it encounters an `<img>`
/// element and uses the real dimensions (and vary key) when available.
#[derive(Debug, Clone, Default)]
pub struct ImageContext {
    /// Map from image source URL to its known metadata.
    known: HashMap<String, ImageMeta>,
}

impl ImageContext {
    /// Creates an empty `ImageContext` with no known images.
    pub fn new() -> Self {
        Self {
            known: HashMap::new(),
        }
    }

    /// Creates an `ImageContext` pre-populated with the given dimension map.
    ///
    /// Vary keys default to empty strings.  Use [`insert`](Self::insert) or
    /// [`insert_with_vary`](Self::insert_with_vary) to set them afterwards.
    pub fn with_dimensions(dimensions: HashMap<String, (f32, f32)>) -> Self {
        let known = dimensions
            .into_iter()
            .map(|(src, (w, h))| {
                (
                    src,
                    ImageMeta {
                        width: w,
                        height: h,
                        vary_key: String::new(),
                    },
                )
            })
            .collect();
        Self { known }
    }

    /// Creates an `ImageContext` pre-populated with a full metadata map
    /// (dimensions *and* vary keys).
    pub fn with_meta(known: HashMap<String, ImageMeta>) -> Self {
        Self { known }
    }

    /// Records intrinsic dimensions for an image source URL (vary key defaults
    /// to empty).
    pub fn insert(&mut self, src: impl Into<String>, width: f32, height: f32) {
        let key = src.into();
        self.known
            .entry(key)
            .and_modify(|m| {
                m.width = width;
                m.height = height;
            })
            .or_insert(ImageMeta {
                width,
                height,
                vary_key: String::new(),
            });
    }

    /// Records intrinsic dimensions **and** a vary key for an image source URL.
    pub fn insert_with_vary(
        &mut self,
        src: impl Into<String>,
        width: f32,
        height: f32,
        vary_key: impl Into<String>,
    ) {
        self.known.insert(
            src.into(),
            ImageMeta {
                width,
                height,
                vary_key: vary_key.into(),
            },
        );
    }

    /// Sets (or overwrites) the vary key for an already-known image.
    ///
    /// If `src` is not yet in the context this is a no-op.
    pub fn set_vary_key(&mut self, src: &str, vary_key: impl Into<String>) {
        if let Some(meta) = self.known.get_mut(src) {
            meta.vary_key = vary_key.into();
        }
    }

    /// Looks up dimensions for the given image source URL.
    ///
    /// Returns `Some((width, height))` when the image has been decoded, or
    /// `None` if the image is still pending / unknown.
    pub fn get(&self, src: &str) -> Option<(f32, f32)> {
        self.known.get(src).map(|m| (m.width, m.height))
    }

    /// Looks up the full [`ImageMeta`] for the given image source URL.
    pub fn get_meta(&self, src: &str) -> Option<&ImageMeta> {
        self.known.get(src)
    }

    /// Returns `true` if there are no known images stored.
    pub fn is_empty(&self) -> bool {
        self.known.is_empty()
    }

    /// Returns how many images are currently stored.
    pub fn len(&self) -> usize {
        self.known.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_context() {
        let ctx = ImageContext::new();
        assert!(ctx.is_empty());
        assert_eq!(ctx.len(), 0);
        assert_eq!(ctx.get("https://example.com/img.png"), None);
        assert!(ctx.get_meta("https://example.com/img.png").is_none());
    }

    #[test]
    fn test_insert_and_get() {
        let mut ctx = ImageContext::new();
        ctx.insert("https://example.com/img.png", 640.0, 480.0);

        assert!(!ctx.is_empty());
        assert_eq!(ctx.len(), 1);
        assert_eq!(ctx.get("https://example.com/img.png"), Some((640.0, 480.0)));

        let meta = ctx.get_meta("https://example.com/img.png").unwrap();
        assert_eq!(meta.width, 640.0);
        assert_eq!(meta.height, 480.0);
        assert!(meta.vary_key.is_empty());
    }

    #[test]
    fn test_insert_with_vary() {
        let mut ctx = ImageContext::new();
        ctx.insert_with_vary("img.png", 100.0, 200.0, "Accept-Encoding");

        let meta = ctx.get_meta("img.png").unwrap();
        assert_eq!(meta.width, 100.0);
        assert_eq!(meta.height, 200.0);
        assert_eq!(meta.vary_key, "Accept-Encoding");
    }

    #[test]
    fn test_set_vary_key() {
        let mut ctx = ImageContext::new();
        ctx.insert("img.png", 100.0, 200.0);
        assert!(ctx.get_meta("img.png").unwrap().vary_key.is_empty());

        ctx.set_vary_key("img.png", "Accept");
        assert_eq!(ctx.get_meta("img.png").unwrap().vary_key, "Accept");
    }

    #[test]
    fn test_set_vary_key_noop_for_unknown() {
        let mut ctx = ImageContext::new();
        ctx.set_vary_key("unknown.png", "Accept");
        assert!(ctx.get_meta("unknown.png").is_none());
    }

    #[test]
    fn test_with_dimensions() {
        let mut map = HashMap::new();
        map.insert("a.png".to_string(), (100.0, 200.0));
        map.insert("b.png".to_string(), (300.0, 400.0));

        let ctx = ImageContext::with_dimensions(map);
        assert_eq!(ctx.len(), 2);
        assert_eq!(ctx.get("a.png"), Some((100.0, 200.0)));
        assert_eq!(ctx.get("b.png"), Some((300.0, 400.0)));
        assert_eq!(ctx.get("c.png"), None);
        assert!(ctx.get_meta("a.png").unwrap().vary_key.is_empty());
    }

    #[test]
    fn test_with_meta() {
        let mut map = HashMap::new();
        map.insert(
            "a.png".to_string(),
            ImageMeta {
                width: 10.0,
                height: 20.0,
                vary_key: "Accept".to_string(),
            },
        );

        let ctx = ImageContext::with_meta(map);
        assert_eq!(ctx.len(), 1);
        let meta = ctx.get_meta("a.png").unwrap();
        assert_eq!(meta.width, 10.0);
        assert_eq!(meta.height, 20.0);
        assert_eq!(meta.vary_key, "Accept");
    }

    #[test]
    fn test_overwrite() {
        let mut ctx = ImageContext::new();
        ctx.insert("img.png", 100.0, 100.0);
        assert_eq!(ctx.get("img.png"), Some((100.0, 100.0)));

        ctx.insert("img.png", 200.0, 300.0);
        assert_eq!(ctx.get("img.png"), Some((200.0, 300.0)));
        assert_eq!(ctx.len(), 1);
    }

    #[test]
    fn test_insert_preserves_vary_key() {
        let mut ctx = ImageContext::new();
        ctx.insert_with_vary("img.png", 100.0, 100.0, "Accept");
        assert_eq!(ctx.get_meta("img.png").unwrap().vary_key, "Accept");

        ctx.insert("img.png", 200.0, 300.0);
        let meta = ctx.get_meta("img.png").unwrap();
        assert_eq!(meta.width, 200.0);
        assert_eq!(meta.height, 300.0);
        assert_eq!(meta.vary_key, "Accept");
    }
}
