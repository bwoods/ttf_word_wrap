//! Useful when creating new wrapping iterators.
use std::str::Chars;

use ttf_parser::Face;

/// A char and it's display width, along wtith the `font_face` and `&str` it came from.
#[derive(Clone, Debug)]
pub struct CharWidth<'a> {
    pub font_face: &'a Face<'a>,
    pub text: &'a str,
    pub ch: char,
    pub display_width: u16,
}

pub trait WithCharWidth {
    fn with_char_width<'a>(&'a self, font_face: &'a Face<'a>) -> CharWidthIterator<'a>;
}

impl WithCharWidth for str {
    fn with_char_width<'a>(&'a self, font_face: &'a Face<'a>) -> CharWidthIterator<'a> {
        let chars = self.chars();
        CharWidthIterator {
            font_face,
            chars,
            text: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CharWidthIterator<'a> {
    font_face: &'a Face<'a>,
    chars: Chars<'a>,
    text: &'a str,
}

impl<'a> Iterator for CharWidthIterator<'a> {
    type Item = CharWidth<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.chars.next()?;

        let width = self
            .font_face
            .glyph_index(ch)
            .map(|glyph_index| self.font_face.glyph_hor_advance(glyph_index))
            .flatten()
            .unwrap_or_default();

        Some(CharWidth {
            ch,
            display_width: width,
            font_face: self.font_face,
            text: self.text,
        })
    }
}
