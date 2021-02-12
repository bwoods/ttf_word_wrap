use crate::{
    grapheme_width::{GraphemeWidthIterator, WithGraphemeWidth},
    line::{LineIterator, Lines},
    line_break::{AddNewlines, LineBreakIterator},
    partial_tokens::{PartialTokensIterator, WithPartialTokens},
    position::{PositionIterator, Positions},
    whitespace::{TokenizeWhiteSpace, WhiteSpaceIterator},
    wordwrap::{WordWrap, WordWrapWithPosition},
    Measure,
};

/// WordWrap for variable-width TTF text.
#[derive(Debug)]
pub struct WhiteSpaceWordWrap<'fnt> {
    max_width: u32,
    measure: &'fnt dyn Measure,
}

impl<'fnt> WhiteSpaceWordWrap<'fnt> {
    /// Creates a new `WhiteSpaceWordWrap`
    ///
    /// Will wrap at `max_width` and measure the glyphs using `font_face`
    pub fn new(max_width: u32, measure: &'fnt dyn Measure) -> Self {
        Self { max_width, measure }
    }
}

impl<'m, 'txt: 'm> WordWrap<'m, 'txt> for WhiteSpaceWordWrap<'m> {
    type Iterator = LineIterator<
        'txt,
        LineBreakIterator<
            PartialTokensIterator<'m, WhiteSpaceIterator<'m, GraphemeWidthIterator<'m>>>,
        >,
    >;

    fn word_wrap(&self, text: &'txt str) -> Self::Iterator {
        text.with_grapheme_width(self.measure)
            .tokenize_white_space(self.measure)
            .with_partial_tokens(self.max_width, text, self.measure)
            .add_newlines_at(self.max_width)
            .lines(text)
    }
}

impl<'m, 'txt: 'm> WordWrapWithPosition<'m, 'txt> for WhiteSpaceWordWrap<'m> {
    type Iterator = PositionIterator<
        'm,
        LineBreakIterator<
            PartialTokensIterator<'m, WhiteSpaceIterator<'m, GraphemeWidthIterator<'m>>>,
        >,
    >;

    fn word_wrap_with_position(&'m self, text: &'txt str) -> Self::Iterator {
        text.with_grapheme_width(self.measure)
            .tokenize_white_space(self.measure)
            .with_partial_tokens(self.max_width, text, self.measure)
            .add_newlines_at(self.max_width)
            .positions(text, self.measure)
    }
}
