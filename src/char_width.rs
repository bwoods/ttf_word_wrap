use std::str::Chars;

use ttf_parser::Face;

/// A char and it's rendered width
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct CharWidth {
    pub ch: char,
    pub width: u32,
}

pub trait WithCharWidth {
    fn with_char_width<'a>(&'a self, font_face: Face<'a>) -> CharWidthIterator<'a>;
}

impl WithCharWidth for str {
    fn with_char_width<'a>(&'a self, font_face: Face<'a>) -> CharWidthIterator<'a> {
        let chars = self.chars();
        CharWidthIterator { font_face, chars }
    }
}

pub struct CharWidthIterator<'a> {
    font_face: Face<'a>,
    chars: Chars<'a>,
}

impl<'a> Iterator for CharWidthIterator<'a> {
    type Item = CharWidth;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.chars.next()?;

        let width = self
            .font_face
            .glyph_index(ch)
            .map(|glyph_index| self.font_face.glyph_hor_advance(glyph_index))
            .flatten()
            .unwrap_or_default() as u32;

        Some(CharWidth { ch, width })
    }
}
