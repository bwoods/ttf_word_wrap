use ttf_parser::Face;

use crate::{
    char_width::{CharWidthIterator, WithCharWidth},
    line_builder::{LineIterator, Lines},
    partial_tokens::{PartialTokensIterator, WithPartialTokens},
    tokenize::whitespace::{TokenizeWhiteSpace, WhiteSpaceIterator},
};

pub trait WordWrap<'fnt, 'txt: 'fnt> {
    type Iterator: 'fnt;
    fn word_wrap(&'fnt self, text: &'txt str) -> Self::Iterator;
}

pub trait Wrap<'fnt, 'txt: 'fnt, T>
where
    T: WordWrap<'fnt, 'txt>,
{
    fn wrap(&self, word_wrap: &'fnt T) -> T::Iterator;
}

impl<'fnt, 'txt: 'fnt, T> Wrap<'fnt, 'txt, T> for &str
where
    T: WordWrap<'fnt, 'txt>,
    T::Iterator: 'fnt,
    Self: 'txt,
{
    fn wrap(&self, word_wrap: &'fnt T) -> T::Iterator {
        word_wrap.word_wrap(self)
    }
}

/// WordWrap for variable-width TTF text.
#[derive(Debug)]
pub struct WhiteSpaceWordWrap<'fnt> {
    max_width: u32,
    font_face: &'fnt Face<'fnt>,
}

impl<'fnt> WhiteSpaceWordWrap<'fnt> {
    pub fn new(max_width: u32, font_face: &'fnt Face<'fnt>) -> Self {
        Self {
            max_width,
            font_face,
        }
    }
}

impl<'fnt, 'txt: 'fnt> WordWrap<'fnt, 'txt> for WhiteSpaceWordWrap<'fnt> {
    type Iterator = LineIterator<
        PartialTokensIterator<'fnt, WhiteSpaceIterator<'fnt, CharWidthIterator<'fnt>>>,
    >;

    fn word_wrap(&self, text: &'txt str) -> Self::Iterator {
        text.with_char_width(self.font_face)
            .tokenize_white_space()
            .with_partial_tokens(self.max_width)
            .lines(self.max_width)
    }
}
