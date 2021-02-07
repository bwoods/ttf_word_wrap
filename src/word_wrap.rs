use crate::{line_builder::LineBuilder, DefaultLineIterator, Options};

use ttf_parser::Face;

/// WordWrap for variable-width TTF text.
#[derive(Debug)]
pub struct WordWrap<'a> {
    font_face: Face<'a>,
}

impl<'a> WordWrap<'a> {
    /// Creates a new `WordWrap` for the given font `Face`
    pub fn new(font_face: Face<'a>) -> Self {
        Self { font_face }
    }

    /// Wraps the given text based on the `options` provided.
    ///
    /// Returns a `LineIterator` that provides the wrapped lines as `&str`.
    pub fn wrap<'b: 'a>(
        &'a self,
        text: &'b str,
        mut options: Options,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let max_width = options.max_width();
        let tokens = options.tokenizer().tokenize(text);
        let iterator = options.line_builder().build(max_width, text, tokens);
        iterator
    }
}
