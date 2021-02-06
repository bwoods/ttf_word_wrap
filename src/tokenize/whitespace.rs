use std::{iter::Peekable, str::Chars};

use super::{span::Span, Token, TokenKind, Tokenizer};

#[derive(Debug, Default)]
pub struct WhiteSpaceTokenizer {}

impl WhiteSpaceTokenizer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Tokenizer for WhiteSpaceTokenizer {
    fn tokenize<'a>(&mut self, text: &'a str) -> Box<dyn Iterator<Item = Token> + 'a> {
        Box::new(WhiteSpaceIterator::new(text))
    }
}

struct WhiteSpaceIterator<'a> {
    chars: Peekable<Chars<'a>>,
    index: usize,
}

impl<'a> WhiteSpaceIterator<'a> {
    fn new(text: &'a str) -> Self {
        let chars = text.chars().peekable();
        Self { chars, index: 0 }
    }
}

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

impl<'a> Iterator for WhiteSpaceIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let state = State::from(self.chars.peek()?);

        // keep track of the start of the span
        let start = self.index;
        let mut end = self.index;

        while let Some(ch) = self.chars.peek() {
            if state != State::from(ch) {
                break;
            }

            // The char is the same 'mode', advance the iterator
            let ch = self.chars.next().unwrap();

            // Increment the end of the span
            end += ch.len_utf8();
        }

        // Done scanning, prep for the next token
        self.index = end;

        let kind = TokenKind::from(state);
        let span = Span::new(start, end);

        Some(Token { span, kind })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_word() {
        let mut iter = WhiteSpaceIterator::new("word");
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 0, end: 4 }
            })
        ));
    }

    #[test]
    fn begin_rn() {
        let mut iter = WhiteSpaceIterator::new("\r\na\n");
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Newline,
                span: Span { start: 0, end: 2 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 2, end: 3 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Newline,
                span: Span { start: 3, end: 4 }
            })
        ));
    }

    #[test]
    fn newline_breaks_ws() {
        let mut iter = WhiteSpaceIterator::new("  \n  ");
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Optional,
                span: Span { start: 0, end: 2 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Newline,
                span: Span { start: 2, end: 3 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Optional,
                span: Span { start: 3, end: 5 }
            })
        ));
    }
    #[test]
    fn mixed() {
        let mut iter = WhiteSpaceIterator::new("at newline\n  some thing");
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 0, end: 2 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Optional,
                span: Span { start: 2, end: 3 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 3, end: 10 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Newline,
                span: Span { start: 10, end: 11 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Optional,
                span: Span { start: 11, end: 13 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 13, end: 17 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Optional,
                span: Span { start: 17, end: 18 }
            })
        ));
        assert!(matches!(
            iter.next(),
            Some(Token {
                kind: TokenKind::Required,
                span: Span { start: 18, end: 23 }
            })
        ));
    }
}
