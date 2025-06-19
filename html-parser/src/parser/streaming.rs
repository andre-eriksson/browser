use api::{collector::Collector, dom::DomNode};
use std::io::BufRead;

use crate::{
    parser::builder::HtmlStreamParserBuilder, tokens::tokenizer::HtmlTokenizer,
    tree::builder::DomTreeBuilder,
};

/// Represents the result of parsing an HTML document.
///
/// # Fields
/// * `dom_tree` - A vector of shared DOM nodes representing the parsed document structure.
/// * `metadata` - The metadata collected during parsing, which is of type `M`.
pub struct ParseResult<M> {
    pub dom_tree: DomNode,
    pub metadata: M,
}

/// A streaming HTML parser that reads HTML content from a buffered reader and builds a DOM tree incrementally.
///
/// # Type Parameters
/// `R` - The type of the buffered reader, which must implement `BufRead`.
/// `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
///
/// # Fields
/// * `reader` - A buffered reader that provides the HTML content to be parsed.
/// * `collector` - An instance of the collector used to gather metadata during parsing.
/// * `buffer` - A string buffer that temporarily holds HTML content between reads.
/// * `buffer_size` - The size of the internal buffer used for reading HTML content.
/// * `byte_buffer` - A vector of bytes that holds any incomplete UTF-8 sequences between reads.
/// * `builder` - An instance of `DomTreeBuilder` that constructs the DOM tree from the parsed tokens.
pub struct HtmlStreamParser<R: BufRead, C: Collector> {
    reader: R,
    collector: C,
    buffer: String,
    buffer_size: usize,
    byte_buffer: Vec<u8>,
}

impl<R: BufRead, C: Collector + Default> HtmlStreamParser<R, C> {
    /// Creates a new `StreamingParser` with the specified reader, collector, and an optional buffer size.
    ///
    /// # Arguments
    /// * `reader` - A buffered reader that implements the `BufRead` trait.
    /// * `collector` - An instance of the collector used to gather metadata during parsing.
    /// * `buffer_size` - An optional size for the internal buffer; if `None`, defaults to 8192 bytes.
    ///
    /// # Returns
    /// A new instance of `StreamingParser` initialized with the provided reader, collector, and buffer size.
    pub fn with_collector(reader: R, collector: C, buffer_size: Option<usize>) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        Self {
            reader,
            collector,
            buffer: String::with_capacity(buffer_size),
            buffer_size: buffer_size,
            byte_buffer: Vec::new(),
        }
    }

    /// Initializes a new `StreamingParserBuilder` with the specified buffered reader and a default collector.
    ///
    /// # Arguments
    /// * `reader` - A buffered reader that implements the `BufRead` trait.
    ///
    /// # Returns
    /// A new instance of `StreamingParserBuilder` initialized with the provided reader and a default collector.
    pub fn builder(reader: R) -> HtmlStreamParserBuilder<R, C> {
        HtmlStreamParserBuilder {
            reader,
            collector: C::default(),
            buffer_size: None,
        }
    }

    /// Initiates the parsing process, reading from the buffered reader and building the DOM tree, by streaming the HTML content.
    ///
    /// # Returns
    /// A `Result` containing a `ParseResult` with the DOM tree and collected metadata, or an error message if parsing fails.
    pub fn parse(mut self) -> Result<ParseResult<C::Output>, String> {
        let mut buf = vec![0u8; self.buffer_size];

        let mut tokenizer = HtmlTokenizer::new();
        let mut builder: DomTreeBuilder<C> = DomTreeBuilder::new();

        while let Ok(bytes_read) = self.reader.read(&mut buf) {
            if bytes_read == 0 {
                break; // EOF
            }

            // Combine any leftover bytes from previous chunk with new data
            let mut combined_bytes = self.byte_buffer.clone();
            combined_bytes.extend_from_slice(&buf[..bytes_read]);

            // Try to convert to UTF-8, handling incomplete sequences
            let (chunk, remaining_bytes) = match self.try_decode_utf8(&combined_bytes) {
                Ok((text, remaining)) => (text, remaining),
                Err(e) => return Err(e),
            };

            // Store any incomplete bytes for the next iteration
            self.byte_buffer = remaining_bytes;

            if !chunk.is_empty() {
                // Prepend the string buffer to the chunk
                let full_chunk = format!("{}{}", self.buffer, chunk);
                self.buffer.clear();

                self.process_chunk(&full_chunk, &mut tokenizer, &mut builder);
            }
        }

        // Handle any remaining bytes at EOF
        if !self.byte_buffer.is_empty() {
            let remaining_text = String::from_utf8_lossy(&self.byte_buffer);
            if !remaining_text.is_empty() {
                let full_chunk = format!("{}{}", self.buffer, remaining_text);
                self.buffer.clear();
                self.process_chunk(&full_chunk, &mut tokenizer, &mut builder);
            }
        }

        Ok(ParseResult {
            dom_tree: DomNode::Document(builder.dom_tree),
            metadata: self.collector.into_result(),
        })
    }

    /// Processes a chunk of HTML content, tokenizing it and building the DOM tree.
    ///
    /// # Arguments
    /// * `chunk` - A string slice containing the HTML content to be processed.
    /// * `tokenizer` - A mutable reference to the `HtmlTokenizer` used for tokenizing the HTML content.
    /// * `builder` - A mutable reference to the `DomTreeBuilder` used for constructing the DOM tree.
    fn process_chunk(
        &mut self,
        chunk: &str,
        tokenizer: &mut HtmlTokenizer,
        builder: &mut DomTreeBuilder<C>,
    ) {
        let tokens = tokenizer.tokenize(chunk.as_bytes());

        builder.build_from_tokens(tokens);
    }

    /// Attempts to decode a byte slice as UTF-8, handling incomplete sequences and invalid bytes.
    ///
    /// # Arguments
    /// * `bytes` - A slice of bytes to decode.
    ///
    /// # Returns
    /// A `Result` containing a tuple of the decoded string and any remaining bytes that could not be decoded,
    /// or an error message if decoding fails.
    fn try_decode_utf8(&self, bytes: &[u8]) -> Result<(String, Vec<u8>), String> {
        match str::from_utf8(bytes) {
            Ok(text) => Ok((text.to_string(), Vec::new())),
            Err(error) => {
                let valid_up_to = error.valid_up_to();

                if valid_up_to == 0 && bytes.len() < 4 {
                    // Might be an incomplete sequence at the start, keep all bytes
                    return Ok((String::new(), bytes.to_vec()));
                }

                // We have some valid UTF-8, decode up to the error point
                let valid_text = str::from_utf8(&bytes[..valid_up_to])
                    .map_err(|e| format!("Unexpected UTF-8 error: {}", e))?;

                // Check if we have an incomplete sequence at the end
                let remaining_bytes = &bytes[valid_up_to..];

                if remaining_bytes.len() < 4 && self.could_be_incomplete_utf8(remaining_bytes) {
                    // Keep the incomplete bytes for next chunk
                    Ok((valid_text.to_string(), remaining_bytes.to_vec()))
                } else {
                    // Invalid UTF-8 sequence, use replacement character
                    let mut result = valid_text.to_string();
                    result.push('�'); // U+FFFD replacement character

                    // Skip the invalid byte(s) and continue with remaining
                    let skip_bytes = error.error_len().unwrap_or(1);
                    let remaining = if valid_up_to + skip_bytes < bytes.len() {
                        bytes[valid_up_to + skip_bytes..].to_vec()
                    } else {
                        Vec::new()
                    };

                    Ok((result, remaining))
                }
            }
        }
    }

    /// Checks if the given byte slice could be the start of an incomplete UTF-8 sequence.
    ///
    /// # Arguments
    /// * `bytes` - A slice of bytes to check.
    ///
    /// # Returns
    /// A boolean indicating whether the byte slice could be an incomplete UTF-8 sequence.
    fn could_be_incomplete_utf8(&self, bytes: &[u8]) -> bool {
        if bytes.is_empty() {
            return false;
        }

        let first_byte = bytes[0];

        // Check if this could be the start of a multi-byte sequence
        if first_byte & 0x80 == 0 {
            false
        } else if first_byte & 0xE0 == 0xC0 {
            bytes.len() < 2
        } else if first_byte & 0xF0 == 0xE0 {
            bytes.len() < 3
        } else if first_byte & 0xF8 == 0xF0 {
            bytes.len() < 4
        } else {
            false
        }
    }
}
