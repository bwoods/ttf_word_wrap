use std::{char, iter::Peekable};

use crate::{char_width::CharWidth, measure::Measure, token::Token, token::TokenKind};

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
    T: Iterator<Item = CharWidth> + Clone,
{
    fn tokenize_white_space(self, measure: &'a dyn Measure) -> WhiteSpaceIterator<'a, T>;
}

impl<'a, T> TokenizeWhiteSpace<'a, T> for T
where
    T: Iterator<Item = CharWidth> + Clone,
{
    fn tokenize_white_space(self, measure: &'a dyn Measure) -> WhiteSpaceIterator<'a, T> {
        WhiteSpaceIterator::new(self.peekable(), measure)
    }
}

#[derive(Clone, Debug)]
pub struct WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth> + Clone,
{
    index: usize,
    chars: Peekable<T>,
    measure: &'a dyn Measure,
}

impl<'a, T> WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth> + Clone,
{
    pub fn new(chars: Peekable<T>, measure: &'a dyn Measure) -> Self {
        Self {
            chars,
            index: 0,
            measure,
        }
    }
}

impl<'a, T> Iterator for WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = CharWidth> + Clone,
{
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let char_width = self.chars.peek()?;
        let state = State::from(&char_width.ch);

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

            // Break at the end of a \r\n, we do not combine multiple newlines
            if state == State::Newline && char_width.ch == '\n' {
                break;
            }
        }

        self.index = end;

        let token = Token {
            start,
            end,
            display_width: total_width,
        };

        match state {
            State::Newline => TokenKind::Newline(Some(token)),
            State::WhiteSpace => TokenKind::Optional(token),
            State::Other => TokenKind::Required(token),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {

    use ttf_parser::Face;

    use crate::{char_width::WithCharWidth, measure::TTFParserMeasure};

    use super::*;

    #[test]
    fn one_word() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "word";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["word"]);

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn sequential_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\r\n\n\r\n\n";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["\r\n", "\n", "\r\n", "\n"]);

        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn begin_rn() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\r\na\n";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["\r\n", "a", "\n"]);

        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn newline_breaks_ws() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "  \n  ";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["  ", "\n", "  "]);

        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn mixed() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "at newline\n  some thing";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
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
        assert!(iter.next().is_none());
    }

    #[test]
    fn with_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "123\n456\r\n7890";
        let mut iter = WhiteSpaceIterator::new(text.with_char_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["123", "\n", "456", "\r\n", "7890"]);

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(iter.next().is_none());
    }
}
