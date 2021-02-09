use ttf_parser::Face;

use crate::{
    char_width::WithCharWidth,
    line_builder::{DefaultLineBuilder, LineBuilder},
    tokenize::whitespace::TokenizeWhiteSpace,
    wrap_at::WrapAt,
};

/// WordWrap for variable-width TTF text.
#[derive(Debug)]
pub struct WordWrap {}

impl WordWrap {
    /// Wraps the given text based on the `options` provided.
    ///
    /// Returns a `LineIterator` that provides the wrapped lines as `&str`.
    pub fn wrap<'a>(
        text: &'a str,
        max_width: u32,
        font_face: Face<'a>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let tokens = text.with_char_width(font_face).tokenize_white_space();
        let max_width_tokens = Box::new(WrapAt::new(max_width, tokens));
        let line_builder = DefaultLineBuilder::new();
        line_builder.build(max_width, max_width_tokens)
    }
}
