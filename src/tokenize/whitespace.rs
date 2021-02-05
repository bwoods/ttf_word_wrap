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

impl<'a> Iterator for WhiteSpaceIterator<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let mode = self.chars.peek()?.is_whitespace();

        // keep track of the start of the span
        let start = self.index;
        let mut end = self.index;

        while let Some(ch) = self.chars.peek() {
            if mode != ch.is_whitespace() {
                break;
            }

            // The char is the same 'mode', advance the iterator
            let ch = self.chars.next().unwrap();

            // Increment the end of the span
            end += ch.len_utf8();
        }

        let span = Span::new(start, end);
        let kind = if mode {
            TokenKind::Optional
        } else {
            TokenKind::Required
        };

        self.index = end + 1;

        Some(Token { span, kind })
    }
}
