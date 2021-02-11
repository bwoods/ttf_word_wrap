//! Useful when creating new wrapping iterators.
use std::str::Chars;

use crate::display_width::Measure;

/// A char and it's display width, along wtith the `font_face` and `&str` it came from.
#[derive(Clone, Debug)]
pub struct CharWidth {
    pub ch: char,
    pub display_width: u16,
}

pub trait WithCharWidth {
    fn with_char_width<'a>(&'a self, display_width: &'a dyn Measure) -> CharWidthIterator<'a>;
}

impl WithCharWidth for str {
    fn with_char_width<'a>(&'a self, display_width: &'a dyn Measure) -> CharWidthIterator<'a> {
        let chars = self.chars();
        CharWidthIterator {
            display_width,
            chars,
        }
    }
}

#[derive(Clone, Debug)]
pub struct CharWidthIterator<'a> {
    display_width: &'a dyn Measure,
    chars: Chars<'a>,
}

impl<'a> Iterator for CharWidthIterator<'a> {
    type Item = CharWidth;

    fn next(&mut self) -> Option<Self::Item> {
        let ch = self.chars.next()?;

        let width = self.display_width.char(ch);

        Some(CharWidth {
            ch,
            display_width: width,
        })
    }
}
