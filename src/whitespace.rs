use std::{char, iter::Peekable};

use crate::{
    char_width::{CharWidth, CharWidthIterator, WithCharWidth},
    display_width::DisplayWidth,
    line::{LineIterator, Lines},
    partial_tokens::{PartialTokensIterator, WithPartialTokens},
    token::Token,
    token::TokenKind,
    word_wrap::{WordWrap, WordWrapWithPosition},
};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Newline,
    WhiteSpace,
    Other,
}

impl From<&char> for State {
    fn from(ch: &char) -> Self {
        if ch == &'\r' || ch == &'\n' {
            State::Newline
        } else if ch.is_whitespace() {
            State::WhiteSpace
        } else {
            State::Other
        }
    }
}

pub trait TokenizeWhiteSpace<'a, T>
where
    T: Iterator<Item = CharWidth<'a>> + Clone,
{
    fn tokenize_white_space(self, display_width: &'a dyn DisplayWidth)
        -> WhiteSpaceIterator<'a, T>;
}

impl<'a, T> TokenizeWhiteSpace<'a, T> for T
where
    T: Iterator<Item = CharWidth<'a>> + Clone,
{
    fn tokenize_white_space(
        self,
        display_width: &'a dyn DisplayWidth,
    ) -> WhiteSpaceIterator<'a, T> {
        WhiteSpaceIterator::new(self.peekable(), display_width)
    }
}

#[derive(Clone, Debug)]
pub struct WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth<'a>> + Clone,
{
    index: usize,
    chars: Peekable<T>,
    display_width: &'a dyn DisplayWidth,
}

impl<'a, T> WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth<'a>> + Clone,
{
    pub fn new(chars: Peekable<T>, display_width: &'a dyn DisplayWidth) -> Self {
        Self {
            chars,
            index: 0,
            display_width,
        }
    }
}

impl<'a, T> Iterator for WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth<'a>> + Clone,
{
    type Item = TokenKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let char_width = self.chars.peek()?;
        let text = char_width.text;
        let state = State::from(&char_width.ch);
        let display_width = self.display_width;

        // keep track of the start of the span
        let start = self.index as usize;
        let mut end = self.index as usize;

        let mut widths = Vec::new();

        // The width of the characters
        let mut total_width: u32 = 0;

        while let Some(char_width) = self.chars.peek() {
            if state != State::from(&char_width.ch) {
                break;
            }

            // The char is the same 'mode', advance the iterator
            let char_width = self.chars.next().unwrap();

            // Increment the end of the span
            end += char_width.ch.len_utf8();
            total_width += u32::from(char_width.display_width);
            widths.push(char_width.display_width);
        }

        self.index = end;

        let token = Token {
            display_width,
            text,
            start,
            end,
            width: total_width,
        };

        match state {
            State::Newline => TokenKind::Newline(token),
            State::WhiteSpace => TokenKind::Optional(token),
            State::Other => TokenKind::Required(token),
        }
        .into()
    }
}

/// WordWrap for variable-width TTF text.
#[derive(Debug)]
pub struct WhiteSpaceWordWrap<'fnt> {
    max_width: u32,
    display_width: &'fnt dyn DisplayWidth,
}

impl<'fnt> WhiteSpaceWordWrap<'fnt> {
    /// Creates a new `WhiteSpaceWordWrap`
    ///
    /// Will wrap at `max_width` and measure the glyphs using `font_face`
    pub fn new(max_width: u32, display_width: &'fnt dyn DisplayWidth) -> Self {
        Self {
            max_width,
            display_width,
        }
    }
}

impl<'fnt, 'txt: 'fnt> WordWrap<'fnt, 'txt> for WhiteSpaceWordWrap<'fnt> {
    type Iterator = LineIterator<
        PartialTokensIterator<'fnt, WhiteSpaceIterator<'fnt, CharWidthIterator<'fnt>>>,
    >;

    fn word_wrap(&self, text: &'txt str) -> Self::Iterator {
        text.with_char_width(self.display_width)
            .tokenize_white_space(self.display_width)
            .with_partial_tokens(self.max_width)
            .lines(self.max_width)
    }
}

impl<'fnt, 'txt: 'fnt> WordWrapWithPosition<'fnt, 'txt> for WhiteSpaceWordWrap<'fnt> {
    type Iterator = LineIterator<
        PartialTokensIterator<'fnt, WhiteSpaceIterator<'fnt, CharWidthIterator<'fnt>>>,
    >;

    fn word_wrap_with_position(&'fnt self, text: &'txt str) -> Self::Iterator {
        text.with_char_width(self.display_width)
            .tokenize_white_space(self.display_width)
            .with_partial_tokens(self.max_width)
            .lines(self.max_width)
    }
}

#[cfg(test)]
mod tests {

    use ttf_parser::Face;

    use crate::{char_width::WithCharWidth, display_width::TTFParserDisplayWidth};

    use super::*;

    #[test]
    fn one_word() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "word";
        let mut iter = WhiteSpaceIterator::new(
            text.with_char_width(&display_width).peekable(),
            &display_width,
        );

        let words: Vec<String> = iter.clone().map(|t| t.into_token().to_string()).collect();
        assert_eq!(words, vec!["word"]);

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
    }

    #[test]
    fn begin_rn() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "\r\na\n";
        let mut iter = WhiteSpaceIterator::new(
            text.with_char_width(&display_width).peekable(),
            &display_width,
        );

        let words: Vec<String> = iter.clone().map(|t| t.into_token().to_string()).collect();
        assert_eq!(words, vec!["\r\n", "a", "\n"]);

        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
    }

    #[test]
    fn newline_breaks_ws() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "  \n  ";
        let mut iter = WhiteSpaceIterator::new(
            text.with_char_width(&display_width).peekable(),
            &display_width,
        );

        let words: Vec<String> = iter.clone().map(|t| t.into_token().to_string()).collect();
        assert_eq!(words, vec!["  ", "\n", "  "]);

        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
    }

    #[test]
    fn mixed() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "at newline\n  some thing";
        let mut iter = WhiteSpaceIterator::new(
            text.with_char_width(&display_width).peekable(),
            &display_width,
        );

        let words: Vec<String> = iter.clone().map(|t| t.into_token().to_string()).collect();
        assert_eq!(
            words,
            vec!["at", " ", "newline", "\n", "  ", "some", " ", "thing"]
        );

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
    }
}
