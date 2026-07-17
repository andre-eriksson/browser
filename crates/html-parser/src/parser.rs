use std::io::BufRead;
use std::mem;

use crate::{
    errors::HtmlParsingError,
    state::{BlockingCause, ResourceMetadata, Script},
};
use html_dom::{Collector, DefaultCollector, DomTreeBuilder, HtmlTokenizer, Token, TokenState, TokenizerState};
use tracing::trace;

use crate::{
    ResourceType,
    state::{BlockedReason, ParserState},
};

/// A streaming HTML parser that reads HTML content in chunks and builds the DOM tree incrementally.
pub struct HtmlStreamParser<R: BufRead, C: Collector + Default> {
    /// The buffered reader from which HTML content is read.
    reader: R,

    /// The internal string buffer for accumulating HTML content.
    buffer: String,

    /// The internal byte buffer for handling incomplete UTF-8 sequences.
    byte_buffer: Vec<u8>,

    /// The state of the HTML tokenizer.
    tokenizer_state: TokenizerState,

    /// The DOM tree builder that constructs the DOM from tokens.
    builder: Option<DomTreeBuilder<C>>,

    /// The current state of the parser.
    state: ParserState<C>,

    /// The state of the previous token processed.
    previous_token_state: TokenState,

    /// The buffer used for reading bytes from the input stream.
    read_buffer: Vec<u8>,
}

impl<R: BufRead, C: Collector + Default> HtmlStreamParser<R, C> {
    /// The default buffer size for reading from the input stream and accumulating HTML content, set to 8 KB.
    const DEFAULT_BUFFER_SIZE: usize = 1024 * 8;

    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buffer: String::with_capacity(Self::DEFAULT_BUFFER_SIZE),
            byte_buffer: Vec::new(),
            tokenizer_state: TokenizerState::default(),
            builder: Some(DomTreeBuilder::new(None)),
            state: ParserState::default(),
            previous_token_state: TokenState::Data,
            read_buffer: vec![0u8; Self::DEFAULT_BUFFER_SIZE],
        }
    }

    pub fn with_collector(mut self, collector: C) -> Self {
        self.builder = Some(DomTreeBuilder::new(Some(collector)));
        self
    }

    /// Processes the next chunk of HTML content from the input stream.
    ///
    /// # Returns
    /// A `Result` containing the current parser state after processing the chunk, or an error message if an error occurs while reading from the stream.
    ///
    /// # Errors
    /// * `HtmlParsingError::UnableToReadStream` - If an error occurs while reading from the input stream.
    /// * `HtmlParsingError::UnexpectedUtf8Error` - If an unexpected UTF-8 decoding error occurs while processing the input stream.
    pub fn step(&mut self) -> Result<ParserState<C>, HtmlParsingError> {
        match &self.state {
            ParserState::Blocked(_) | ParserState::Completed(_) => return Ok(std::mem::take(&mut self.state)),
            ParserState::Running => {}
        }

        if !self.buffer.is_empty() {
            let full_chunk = mem::take(&mut self.buffer);
            self.process_chunk(&full_chunk);
            if !matches!(self.state, ParserState::Running) {
                return Ok(std::mem::take(&mut self.state));
            }
        }

        match self.reader.read(&mut self.read_buffer) {
            Ok(0) => {
                if !self.byte_buffer.is_empty() {
                    let mut flush_chunk = String::from_utf8_lossy(&self.byte_buffer).into_owned();
                    if !flush_chunk.is_empty() {
                        if !self.buffer.is_empty() {
                            let mut prefix = mem::take(&mut self.buffer);
                            prefix.push_str(&flush_chunk);
                            flush_chunk = prefix;
                        }

                        self.process_chunk(&flush_chunk);

                        if !matches!(self.state, ParserState::Running) {
                            self.byte_buffer.clear();
                            return Ok(std::mem::take(&mut self.state));
                        }
                    }

                    self.byte_buffer.clear();
                }

                if !matches!(self.state, ParserState::Blocked(_))
                    && let Some(builder) = self.builder.take()
                {
                    self.state = ParserState::Completed(builder.finalize());
                }
                Ok(std::mem::take(&mut self.state))
            }
            Ok(bytes_read) => {
                let mut combined_bytes = self.byte_buffer.clone();
                combined_bytes.extend_from_slice(&self.read_buffer[..bytes_read]);

                let (chunk, remaining_bytes) = Self::try_decode_utf8(&combined_bytes)?;
                self.byte_buffer = remaining_bytes;

                if !chunk.is_empty() {
                    if self.buffer.is_empty() {
                        self.process_chunk(&chunk);
                    } else {
                        let mut full_chunk = mem::take(&mut self.buffer);
                        full_chunk.push_str(&chunk);
                        self.process_chunk(&full_chunk);
                    }

                    if !matches!(self.state, ParserState::Running) {
                        return Ok(std::mem::take(&mut self.state));
                    }
                }

                Ok(std::mem::take(&mut self.state))
            }
            Err(e) => Err(HtmlParsingError::UnableToReadStream(e.to_string())),
        }
    }

    /// Extracts content from the input stream until the specified end tag is found.
    ///
    /// # Arguments
    /// * `tag` - The end tag to search for (e.g., `</script>` or `</style>`).
    ///
    /// # Returns
    /// A `Result` containing the extracted content if successful, or an error message if the end tag is not found before the end of the stream or if reading from the stream fails.
    ///
    /// # Errors
    /// * `HtmlParsingError::MalformedDocument` - If the specified end tag is not found before the end of the stream.
    /// * `HtmlParsingError::UnableToReadStream` - If an error occurs while reading from the stream.
    fn extract_content_until_end_tag(&mut self, tag: &str) -> Result<String, HtmlParsingError> {
        let mut content = String::new();
        let tag_lower = tag.to_ascii_lowercase();
        let tail_len = tag_lower.len().saturating_sub(1);

        loop {
            if !self.buffer.is_empty() {
                let buffer_lower = self.buffer.to_ascii_lowercase();

                if let Some(idx) = buffer_lower.find(&tag_lower) {
                    content.push_str(&self.buffer[..idx]);
                    self.buffer = self.buffer[idx + tag.len()..].to_string();
                    return Ok(content);
                }

                if self.buffer.len() > tail_len {
                    let mut split_at = self.buffer.len() - tail_len;
                    while split_at > 0 && !self.buffer.is_char_boundary(split_at) {
                        split_at -= 1;
                    }

                    content.push_str(&self.buffer[..split_at]);
                    self.buffer = self.buffer[split_at..].to_string();
                }
            }

            match self.reader.read(&mut self.read_buffer) {
                Ok(0) => {
                    return Err(HtmlParsingError::MalformedDocument(format!(
                        "End tag '{tag}' not found before end of stream"
                    )));
                }
                Ok(bytes_read) => {
                    let mut combined_bytes = self.byte_buffer.clone();
                    combined_bytes.extend_from_slice(&self.read_buffer[..bytes_read]);

                    let (chunk, remaining_bytes) = Self::try_decode_utf8(&combined_bytes)?;
                    self.byte_buffer = remaining_bytes;

                    if !chunk.is_empty() {
                        self.buffer.push_str(&chunk);
                    }
                }
                Err(e) => {
                    return Err(HtmlParsingError::UnableToReadStream(e.to_string()));
                }
            }
        }
    }

    /// Processes a chunk of HTML content, tokenizing it and updating the parser state.
    ///
    /// # Arguments
    /// * `chunk` - A string slice containing the HTML content to process.
    fn process_chunk(&mut self, chunk: &str) {
        let mut tokens: Vec<Token> = Vec::new();

        for (idx, ch) in chunk.char_indices() {
            HtmlTokenizer::process_char(&mut self.tokenizer_state, ch, &mut tokens);
            let current_state = self.tokenizer_state.state;
            if current_state == self.previous_token_state {
                continue;
            }
            self.previous_token_state = current_state;

            let Some(cause) = BlockingCause::classify_cause(current_state, tokens.last()) else {
                continue;
            };

            let last_token = tokens.last().cloned();
            self.builder
                .as_mut()
                .unwrap()
                .build_from_tokens(mem::take(&mut tokens));

            let next_idx = idx + ch.len_utf8();
            if next_idx < chunk.len() {
                self.buffer.push_str(&chunk[next_idx..]);
            }

            let reason = match cause {
                BlockingCause::Script => {
                    trace!("Blocking parser for script content at token: {:?}", last_token);

                    let attributes = last_token.and_then(|t| t.attributes);

                    let src = attributes
                        .as_ref()
                        .and_then(|attrs| attrs.get("src").cloned());

                    let script = match src {
                        Some(src) => {
                            let is_async = attributes
                                .as_ref()
                                .and_then(|attrs| attrs.get("async").cloned())
                                .is_some_and(|value| {
                                    value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("")
                                });

                            let is_deferred = attributes
                                .as_ref()
                                .and_then(|attrs| attrs.get("defer").cloned())
                                .is_some_and(|value| {
                                    value.eq_ignore_ascii_case("true") || value.eq_ignore_ascii_case("")
                                });

                            Script::External {
                                src,
                                is_async,
                                is_deferred,
                            }
                        }
                        None => {
                            let data = self.extract_content_until_end_tag("</script>");

                            let type_attr = attributes
                                .as_ref()
                                .and_then(|attrs| attrs.get("type").cloned())
                                .unwrap_or_else(|| "text/javascript".to_string());

                            Script::Inline { data, type_attr }
                        }
                    };

                    BlockedReason::WaitingForScript { script }
                }
                BlockingCause::Style => {
                    trace!("Blocking parser for style content at token: {:?}", last_token);

                    let data = self.extract_content_until_end_tag("</style>");
                    let attributes = last_token.and_then(|t| t.attributes);
                    BlockedReason::WaitingForStyle { data, attributes }
                }
                BlockingCause::Svg => {
                    trace!("Blocking parser for SVG content at token: {:?}", last_token);

                    let data = self.extract_content_until_end_tag("</svg>");
                    BlockedReason::SVGContent { data }
                }
                BlockingCause::Math => {
                    trace!("Blocking parser for Math content at token: {:?}", last_token);
                    let data = self.extract_content_until_end_tag("</math>");
                    BlockedReason::MathML { data }
                }
                BlockingCause::Stylesheet { href } => {
                    trace!("Blocking parser for stylesheet resource at token: {:?}", last_token);

                    BlockedReason::WaitingForResource(ResourceType::Style, href, ResourceMetadata::default())
                }
                BlockingCause::Favicon {
                    href,
                    content_type,
                    sizes,
                } => {
                    trace!("Blocking parser for favicon resource at token: {:?}", last_token);

                    BlockedReason::WaitingForResource(
                        ResourceType::Favicon,
                        href,
                        ResourceMetadata {
                            content_type,
                            sizes,
                        },
                    )
                }
            };

            self.state = ParserState::Blocked(reason);
            return;
        }

        self.builder.as_mut().unwrap().build_from_tokens(tokens);
    }

    /// Attempts to decode a byte slice as UTF-8, handling incomplete sequences and invalid bytes.
    ///
    /// # Arguments
    /// * `bytes` - A slice of bytes to decode.
    ///
    /// # Returns
    /// A `Result` containing a tuple of the decoded string and any remaining bytes that could not be decoded,
    /// or an error message if decoding fails.
    fn try_decode_utf8(bytes: &[u8]) -> Result<(String, Vec<u8>), HtmlParsingError> {
        match str::from_utf8(bytes) {
            Ok(text) => Ok((text.to_string(), Vec::new())),
            Err(error) => {
                let valid_up_to = error.valid_up_to();

                if valid_up_to == 0 && bytes.len() < 4 {
                    return Ok((String::new(), bytes.to_vec()));
                }

                let valid_text =
                    str::from_utf8(&bytes[..valid_up_to]).map_err(|e| format!("Unexpected UTF-8 error: {e}"));

                let valid_text = match valid_text {
                    Ok(text) => text,
                    Err(e) => return Err(HtmlParsingError::UnexpectedUtf8Error(e)),
                };

                let remaining_bytes = &bytes[valid_up_to..];

                if remaining_bytes.len() < 4 && Self::could_be_incomplete_utf8(remaining_bytes) {
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
    fn could_be_incomplete_utf8(bytes: &[u8]) -> bool {
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

impl<R: BufRead> HtmlStreamParser<R, DefaultCollector> {
    pub fn simple(reader: R) -> Self {
        Self::new(reader)
    }
}
