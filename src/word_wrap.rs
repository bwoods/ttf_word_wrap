use crate::{LineIterator, Options};

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
    pub fn wrap<'b>(&mut self, text: &'b str, mut options: Options) -> LineIterator<'b> {
        let tokens = options.tokenizer().tokenize(text);
        LineIterator::new(text, options, tokens)
    }
}
