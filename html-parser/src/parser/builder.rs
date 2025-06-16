use std::io::BufRead;

use api::collector::Collector;

use crate::parser::streaming::HtmlStreamParser;

/// A builder for constructing a `StreamingParser` with a specified buffered reader and collector.
///
/// # Type Parameters
/// `R` - The type of the buffered reader, which must implement `BufRead`.
/// `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
///
/// # Fields
/// * `reader` - A buffered reader that provides the HTML content to be parsed.
/// * `collector` - An instance of the collector used to gather metadata during parsing.
/// * `buffer_size` - An optional size for the internal buffer used for reading HTML content; defaults to 8192 bytes if not specified.
pub struct HtmlStreamParserBuilder<R: BufRead, C: Collector> {
    pub reader: R,
    pub collector: C,
    pub buffer_size: Option<usize>,
}

impl<R: BufRead, C: Collector + Default> HtmlStreamParserBuilder<R, C> {
    /// Sets the buffer size for the streaming parser, defaulting to 8192 bytes if not specified.
    ///
    /// # Arguments
    /// * `buffer_size` - The size of the internal buffer used for reading HTML content.
    ///
    /// # Returns
    /// The size of the buffer that will be used for reading HTML content, which is set to the specified value or defaults to 8192 bytes if not set.
    pub fn buffer_size(mut self, buffer_size: usize) -> usize {
        self.buffer_size = Some(buffer_size);
        self.buffer_size.unwrap_or(1024 * 8) // Default to 8192 bytes if not set
    }

    /// Sets the collector for the streaming parser.
    ///
    /// # Arguments
    /// * `collector` - An instance of the collector used to gather metadata during parsing.
    ///
    /// # Returns
    /// A new instance of `StreamingParserBuilder` with the specified collector.
    pub fn collector(mut self, collector: C) -> Self {
        self.collector = collector;
        self
    }

    /// Builds the `StreamingParser` with the specified reader, collector, and buffer size.
    ///
    /// # Returns
    /// A new instance of `StreamingParser` initialized with the provided reader, collector, and buffer size.
    pub fn build(self) -> HtmlStreamParser<R, C> {
        HtmlStreamParser::with_collector(self.reader, self.collector, self.buffer_size)
    }
}
