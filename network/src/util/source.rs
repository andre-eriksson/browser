/// The type of source for a request, such as a script tag or a stylesheet link.
#[derive(Debug, PartialEq)]
pub enum SourceType {
    /// The `src` attribute from an `<iframe>` element
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/iframe#src>
    Frame,

    /// The `src` attribute from a `<script>` element
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/script#src>
    Script,

    /// The `href` attribute from a `<link>` element which has the attribute `rel="stylesheet"`
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/link#href>
    Style,

    /// The `src` or `srcset` attribute from an `<img>` element
    ///
    /// # See Also
    /// * <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img#src>
    /// * <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/img#srcset>
    Image,

    /// The `href` attribute from a `<link>` element which has the attribute `as="font"` or via `@font-face` in a stylesheet
    ///
    /// # See Also
    /// * <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/link#as>
    /// * <https://developer.mozilla.org/en-US/docs/Web/CSS/@font-face>
    Font,

    /// The `src` attribute from a `<audio>` or `<video>` element
    ///
    /// # See Also
    /// * <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/video#src>
    /// * <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/audio#src>
    Media,

    /// The `serviceWorker.register()` method
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/API/ServiceWorkerContainer/register>
    Worker,

    /// The `href` attribute from a `<link>` element which has the attribute `rel="manifest"`
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/link#href>
    Manifest,

    /// The URL in the `window.fetch()` method
    ///
    /// # See Also
    /// <https://developer.mozilla.org/en-US/docs/Web/API/Window/fetch>
    Fetch,
}

/// Converts a tag name to its corresponding `SourceType`.
///
/// # Arguments
/// * `tag_name` - The name of the HTML tag to convert.
///
/// # Returns
/// The corresponding `SourceType` for the given tag name or `SourceType::Fetch` if the tag is unrecognized.
pub fn get_source_from_tag(tag_name: &str) -> SourceType {
    match tag_name {
        "frame" => SourceType::Frame,
        "script" => SourceType::Script,
        "style" => SourceType::Style,
        "img" => SourceType::Image,
        "font" => SourceType::Font,
        "media" => SourceType::Media,
        "worker" => SourceType::Worker,
        "manifest" => SourceType::Manifest,
        _ => SourceType::Fetch, // Default to Fetch for unrecognized tags
    }
}
