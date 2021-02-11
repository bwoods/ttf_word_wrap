//! Useful when creating new wrapping iterators.
use std::str::Chars;

use crate::display_width::DisplayWidth;

/// A char and it's display width, along wtith the `font_face` and `&str` it came from.
#[derive(Clone, Debug)]
pub struct CharWidth<'a> {
    pub text: &'a str,
    pub ch: char,
    pub display_width: u16,
}

pub trait WithCharWidth {
    fn with_char_width<'a>(&'a self, display_width: &'a dyn DisplayWidth) -> CharWidthIterator<'a>;
}

impl WithCharWidth for str {
    fn with_char_width<'a>(&'a self, display_width: &'a dyn DisplayWidth) -> CharWidthIterator<'a> {
        let chars = self.chars();
        CharWidthIterator {
            display_width,
            chars,
            text: self,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CharWidthIterator<'a> {
    display_width: &'a dyn DisplayWidth,
    chars: Chars<'a>,
    text: &'a str,
}

impl<'a> Iterator for CharWidthIterator<'a> {
    type Item = CharWidth<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.chars.next()?;

        let width = self.display_width.measure_char(ch);

        Some(CharWidth {
            ch,
            display_width: width,
            text: self.text,
        })
    }
}
