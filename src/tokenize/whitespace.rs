use std::iter::Peekable;

use crate::char_width::CharWidth;

use super::{Token, TokenKind};

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

impl From<State> for TokenKind {
    fn from(state: State) -> Self {
        match state {
            State::Newline => TokenKind::Newline,
            State::WhiteSpace => TokenKind::Optional,
            State::Other => TokenKind::Required,
        }
    }
}

pub trait TokenizeWhiteSpace<T>
where
    T: Iterator<Item = CharWidth>,
{
    fn tokenize_white_space(self) -> WhiteSpaceIterator<T>;
}

impl<T> TokenizeWhiteSpace<T> for T
where
    T: Iterator<Item = CharWidth>,
{
    fn tokenize_white_space(self) -> WhiteSpaceIterator<T> {
        WhiteSpaceIterator::new(self.peekable())
    }
}

#[derive(Clone)]
pub struct WhiteSpaceIterator<T>
where
    T: Iterator<Item = CharWidth>,
{
    index: usize,
    chars: Peekable<T>,
}

impl<T> WhiteSpaceIterator<T>
where
    T: Iterator<Item = CharWidth>,
{
    pub fn new(chars: Peekable<T>) -> Self {
        Self { chars, index: 0 }
    }
}

impl<T> Iterator for WhiteSpaceIterator<T>
where
    T: Iterator<Item = CharWidth>,
{
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let char_width = self.chars.peek()?;
        let state = State::from(&char_width.ch);

        // keep track of the start of the span
        let start = self.index as usize;
        let mut end = self.index as usize;

        // The width of the characters
        let mut total_width = 0;

        while let Some(char_width) = self.chars.peek() {
            if state != State::from(&char_width.ch) {
                break;
            }

            // The char is the same 'mode', advance the iterator
            let char_width = self.chars.next().unwrap();

            // Increment the end of the span
            end += char_width.ch.len_utf8();
            total_width += char_width.width;
        }

        self.index = end + 1;

        // Done scanning, prep for the next token
        let kind = TokenKind::from(state);

        Some(Token {
            start,
            end,
            kind,
            width: total_width,
        })
    }
}

/*
#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn one_word() {
        use TokenKind::*;

        let var_name: &str = "word";
        let iter = WhiteSpaceIterator::new(var_name);

        let words: Vec<String> = iter.clone().map(|t| t.to_string()).collect();
        assert_eq!(words, vec!["word"]);

        let kinds: Vec<TokenKind> = iter.clone().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![Required]);
    }

    #[test]
    fn begin_rn() {
        use TokenKind::*;

        let mut iter = WhiteSpaceIterator::new("\r\na\n");

        let words: Vec<String> = iter.clone().map(|t| t.to_string()).collect();
        assert_eq!(words, vec!["\r\n", "a", "\n"]);

        let kinds: Vec<TokenKind> = iter.clone().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![Newline, Required, Newline]);
    }

    #[test]
    fn newline_breaks_ws() {
        use TokenKind::*;

        let mut iter = WhiteSpaceIterator::new("  \n  ");

        let words: Vec<String> = iter.clone().map(|t| t.to_string()).collect();
        assert_eq!(words, vec!["  ", "\n", "  "]);

        let kinds: Vec<TokenKind> = iter.clone().map(|t| t.kind).collect();
        assert_eq!(kinds, vec![Optional, Newline, Optional]);
    }
    #[test]
    fn mixed() {
        use TokenKind::*;

        let mut iter = WhiteSpaceIterator::new("at newline\n  some thing");

        let words: Vec<String> = iter.clone().map(|t| t.to_string()).collect();
        assert_eq!(
            words,
            vec!["at", " ", "newline", "\n", "  ", "some", " ", "thing"]
        );

        let kinds: Vec<TokenKind> = iter.clone().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![Required, Optional, Required, Newline, Optional, Required, Optional, Required]
        );
    }
}
*/
