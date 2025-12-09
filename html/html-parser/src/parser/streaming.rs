use std::io::BufRead;

use crate::tokens::tokenizer::{HtmlTokenizer, TokenizerState};
use html_dom::builder::{BuildResult, DomTreeBuilder};
use html_syntax::collector::Collector;

/// A streaming HTML parser that reads HTML content in chunks and builds the DOM tree incrementally.
///
/// # Type Parameters
/// * `R` - The type of the buffered reader, which must implement `BufRead`.
pub struct HtmlStreamParser<R: BufRead> {
    /// A buffered reader that provides the HTML content to be parsed.
    reader: R,

    /// A string buffer that temporarily holds HTML content between reads.
    buffer: String,

    /// The size of the internal buffer used for reading HTML content.
    buffer_size: usize,

    /// A vector of bytes that holds any incomplete UTF-8 sequences between reads.
    byte_buffer: Vec<u8>,

    /// The current state of the HTML tokenizer.
    tokenizer_state: TokenizerState,
}

impl<R: BufRead> HtmlStreamParser<R> {
    /// Creates a new `StreamingParser` with the specified reader, and an optional buffer size.
    ///
    /// # Arguments
    /// * `reader` - A buffered reader that implements the `BufRead` trait.
    /// * `buffer_size` - An optional size for the internal buffer; if `None`, defaults to 8192 bytes.
    ///
    /// # Returns
    /// A new instance of `StreamingParser` initialized with the provided reader, and buffer size.
    pub fn new(reader: R, buffer_size: Option<usize>) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        Self {
            reader,
            buffer: String::with_capacity(buffer_size),
            buffer_size,
            byte_buffer: Vec::new(),
            tokenizer_state: TokenizerState::default(),
        }
    }

    /// Initiates the parsing process, reading from the buffered reader and building the DOM tree, by streaming the HTML content.
    ///
    /// # Type Parameters
    /// * `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
    ///
    /// # Arguments
    /// * `collector` - An optional collector instance that implements the `Collector` trait. If `None`, a default collector is used.
    ///
    /// # Returns
    /// A `Result` containing a `ParseResult` with the DOM tree and collected metadata, or an error message if parsing fails.
    pub fn parse<C: Collector + Default>(
        mut self,
        collector: Option<C>,
    ) -> Result<BuildResult<C::Output>, String> {
        let mut buf = vec![0u8; self.buffer_size];
        let mut builder = DomTreeBuilder::new(Some(collector.unwrap_or_default()));

        while let Ok(bytes_read) = self.reader.read(&mut buf) {
            if bytes_read == 0 {
                break;
            }

            let mut combined_bytes = self.byte_buffer.clone();
            combined_bytes.extend_from_slice(&buf[..bytes_read]);

            let (chunk, remaining_bytes) = match self.try_decode_utf8(&combined_bytes) {
                Ok((text, remaining)) => (text, remaining),
                Err(e) => return Err(e),
            };

            self.byte_buffer = remaining_bytes;

            if !chunk.is_empty() {
                let full_chunk = format!("{}{}", self.buffer, chunk);
                self.buffer.clear();

                self.process_chunk(&full_chunk, &mut builder);
            }
        }

        if !self.byte_buffer.is_empty() {
            let remaining_text = String::from_utf8_lossy(&self.byte_buffer);
            if !remaining_text.is_empty() {
                let full_chunk = format!("{}{}", self.buffer, remaining_text);
                self.buffer.clear();
                self.process_chunk(&full_chunk, &mut builder);
            }
        }

        Ok(builder.finalize())
    }

    /// Processes a chunk of HTML content, tokenizing it and building the DOM tree.
    ///
    /// # Type Parameters
    /// * `C` - The type of the collector used to gather metadata during parsing, which must implement the `Collector` trait.
    ///
    /// # Arguments
    /// * `chunk` - A string slice containing the HTML content to be processed.
    /// * `tokenizer` - A mutable reference to the `HtmlTokenizer` used for tokenizing the HTML content.
    /// * `builder` - A mutable reference to the `DomTreeBuilder` used for constructing the DOM tree.
    fn process_chunk<C: Collector + Default>(
        &mut self,
        chunk: &str,
        builder: &mut DomTreeBuilder<C>,
    ) {
        let mut tokens = Vec::new();

        for ch in chunk.chars() {
            HtmlTokenizer::process_char(&mut self.tokenizer_state, ch, &mut tokens);
        }

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
                    return Ok((String::new(), bytes.to_vec()));
                }

                let valid_text = str::from_utf8(&bytes[..valid_up_to])
                    .map_err(|e| format!("Unexpected UTF-8 error: {}", e))?;

                let remaining_bytes = &bytes[valid_up_to..];

                if remaining_bytes.len() < 4 && self.could_be_incomplete_utf8(remaining_bytes) {
                    Ok((valid_text.to_string(), remaining_bytes.to_vec()))
                } else {
                    let mut result = valid_text.to_string();
                    result.push('�');

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
