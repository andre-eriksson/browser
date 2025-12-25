use std::io::BufRead;
use std::mem;

use html_dom::builder::{BuildResult, DomTreeBuilder};
use html_syntax::{collector::Collector, token::Token};
use html_tokenizer::{
    state::TokenState,
    tokenizer::{HtmlTokenizer, TokenizerState},
};

use crate::state::{BlockedReason, ParserState};

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
    builder: DomTreeBuilder<C>,

    /// The current state of the parser.
    state: ParserState,

    /// The state of the previous token processed.
    previous_token_state: TokenState,

    /// The buffer used for reading bytes from the input stream.
    read_buffer: Vec<u8>,
}

impl<R: BufRead, C: Collector + Default> HtmlStreamParser<R, C> {
    pub fn new(reader: R, buffer_size: Option<usize>, collector: Option<C>) -> Self {
        let buffer_size = buffer_size.unwrap_or(1024 * 8);
        Self {
            reader,
            buffer: String::with_capacity(buffer_size),
            byte_buffer: Vec::new(),
            tokenizer_state: TokenizerState::default(),
            builder: DomTreeBuilder::new(collector),
            state: ParserState::default(),
            previous_token_state: TokenState::Data,
            read_buffer: vec![0u8; buffer_size],
        }
    }

    /// Creates a simple `HtmlStreamParser` with default settings.
    ///
    /// # Arguments
    /// * `reader` - A buffered reader from which HTML content will be read.
    pub fn simple(reader: R) -> Self {
        Self::new(reader, None, None)
    }

    /// Returns a reference to the current state of the parser.
    pub fn get_state(&self) -> &ParserState {
        &self.state
    }

    /// Resumes the parser if it is currently blocked.
    ///
    /// Assumes that the conditions for resuming have been met (e.g., scripts/styles have been handled).
    ///
    /// # Returns
    /// * `Ok(&ParserState)` - A reference to the current parser state after resuming.
    /// * `Err(String)` - An error message if the parser has already completed.
    pub fn resume(&mut self) -> Result<&ParserState, String> {
        match &self.state {
            ParserState::Blocked(_) => {
                self.state = ParserState::Running;
                Ok(&self.state)
            }
            ParserState::Completed => Err("Parser has already completed".to_string()),
            ParserState::Running => Ok(&self.state),
        }
    }

    /// Processes the next chunk of HTML content from the input stream.
    ///
    /// # Returns
    /// * `Ok(&ParserState)` - A reference to the current parser state after processing the chunk.
    /// * `Err(String)` - An error message if reading from the stream fails.
    pub fn step(&mut self) -> Result<&ParserState, String> {
        match &self.state {
            ParserState::Blocked(_) => return Ok(&self.state),
            ParserState::Completed => return Ok(&self.state),
            ParserState::Running => {}
        }

        if !self.buffer.is_empty() {
            let full_chunk = mem::take(&mut self.buffer);
            self.process_chunk(&full_chunk)?;
            if !matches!(self.state, ParserState::Running) {
                return Ok(&self.state);
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

                        self.process_chunk(&flush_chunk)?;

                        if !matches!(self.state, ParserState::Running) {
                            self.byte_buffer.clear();
                            return Ok(&self.state);
                        }
                    }

                    self.byte_buffer.clear();
                }

                if !matches!(self.state, ParserState::Blocked(_)) {
                    self.state = ParserState::Completed;
                }
                Ok(&self.state)
            }
            Ok(bytes_read) => {
                let mut combined_bytes = self.byte_buffer.clone();
                combined_bytes.extend_from_slice(&self.read_buffer[..bytes_read]);

                let (chunk, remaining_bytes) = self.try_decode_utf8(&combined_bytes)?;
                self.byte_buffer = remaining_bytes;

                if !chunk.is_empty() {
                    if !self.buffer.is_empty() {
                        let mut full_chunk = mem::take(&mut self.buffer);
                        full_chunk.push_str(&chunk);
                        self.process_chunk(&full_chunk)?;
                    } else {
                        self.process_chunk(&chunk)?;
                    }

                    if !matches!(self.state, ParserState::Running) {
                        return Ok(&self.state);
                    }
                }

                Ok(&self.state)
            }
            Err(e) => Err(format!("Error reading from stream: {}", e)),
        }
    }

    /// Extracts the content of a `<script>` tag when the parser is blocked waiting for a script.
    ///
    /// # Returns
    /// * `Ok(String)` - The content of the `<script>` tag.
    /// * `Err(String)` - An error message if the parser is not blocked waiting for a script or if extraction fails.
    pub fn extract_script_content(&mut self) -> Result<String, String> {
        if !matches!(
            self.state,
            ParserState::Blocked(BlockedReason::WaitingForScript(_)),
        ) {
            return Err("Parser is not blocked waiting for script".to_string());
        }

        self.extract_content_until_end_tag("</script>")
    }

    /// Extracts the content of a `<style>` tag when the parser is blocked waiting for a style.
    ///
    /// # Returns
    /// * `Ok(String)` - The content of the `<style>` tag.
    /// * `Err(String)` - An error message if the parser is not blocked waiting for a style or if extraction fails.
    pub fn extract_style_content(&mut self) -> Result<String, String> {
        if !matches!(
            self.state,
            ParserState::Blocked(BlockedReason::WaitingForStyle(_)),
        ) {
            return Err("Parser is not blocked waiting for style".to_string());
        }

        self.extract_content_until_end_tag("</style>")
    }

    /// Extracts content from the input stream until the specified end tag is found.
    ///
    /// # Arguments
    /// * `tag` - The end tag to search for (e.g., `</script>` or `</style>`).
    ///
    /// # Returns
    /// * `Ok(String)` - The content found before the end tag.
    /// * `Err(String)` - An error message if the end tag is not found or
    fn extract_content_until_end_tag(&mut self, tag: &str) -> Result<String, String> {
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
                    return Err(format!("End tag '{}' not found before end of stream", tag));
                }
                Ok(bytes_read) => {
                    let mut combined_bytes = self.byte_buffer.clone();
                    combined_bytes.extend_from_slice(&self.read_buffer[..bytes_read]);

                    let (chunk, remaining_bytes) = self.try_decode_utf8(&combined_bytes)?;
                    self.byte_buffer = remaining_bytes;

                    if !chunk.is_empty() {
                        self.buffer.push_str(&chunk);
                    }
                }
                Err(e) => {
                    return Err(format!("Error reading from stream: {}", e));
                }
            }
        }
    }

    /// Finalizes the parsing process and returns the built DOM tree.
    ///
    /// # Returns
    /// A `BuildResult` containing the final output of the DOM tree.
    pub fn finalize(self) -> BuildResult<C> {
        self.builder.finalize()
    }

    /// Processes a chunk of HTML content, tokenizing it and updating the parser state.
    ///
    /// # Arguments
    /// * `chunk` - A string slice containing the HTML content to process.
    ///
    /// # Returns
    /// * `Ok(())` - If the chunk was processed successfully.
    /// * `Err(String)` - An error message if processing fails.
    fn process_chunk(&mut self, chunk: &str) -> Result<(), String> {
        let mut tokens: Vec<Token> = Vec::new();

        for (idx, ch) in chunk.char_indices() {
            HtmlTokenizer::process_char(&mut self.tokenizer_state, ch, &mut tokens);

            if self.previous_token_state != self.tokenizer_state.state {
                let current_state = self.tokenizer_state.state;

                let blocked_reason = match current_state {
                    TokenState::ScriptData => {
                        let attributes = tokens
                            .last()
                            .map(|t| t.attributes.clone())
                            .unwrap_or_default();

                        Some(BlockedReason::WaitingForScript(attributes))
                    }
                    TokenState::StyleData => {
                        let attributes = tokens
                            .last()
                            .map(|t| t.attributes.clone())
                            .unwrap_or_default();

                        Some(BlockedReason::WaitingForStyle(attributes))
                    }
                    _ => None,
                };

                if let Some(reason) = blocked_reason {
                    self.builder.build_from_tokens(mem::take(&mut tokens));

                    let next_idx = idx + ch.len_utf8();
                    if next_idx < chunk.len() {
                        self.buffer.push_str(&chunk[next_idx..]);
                    }

                    self.state = ParserState::Blocked(reason);

                    self.previous_token_state = current_state;
                    return Ok(());
                }

                self.previous_token_state = current_state;
            }
        }

        self.builder.build_from_tokens(tokens);
        Ok(())
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
