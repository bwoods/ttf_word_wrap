use std::{fmt::Formatter, iter::Peekable};

use crate::tokenize::{Token, TokenKind};

pub trait LineBuilder: std::fmt::Debug {
    fn build<'a>(
        &self,
        max_width: u32,
        text: &'a str,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

/// Provides lines as `&str`
#[derive(Debug)]
pub struct DefaultLineBuilder {}

impl DefaultLineBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl LineBuilder for DefaultLineBuilder {
    fn build<'a>(
        &self,
        max_width: u32,
        text: &'a str,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let tokens = tokens.peekable();
        Box::new(DefaultLineIterator {
            max_width,
            text,
            tokens,
        })
    }
}

/// Provides lines as `&str`
pub struct DefaultLineIterator<'a> {
    max_width: u32,
    text: &'a str,
    tokens: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

impl<'a> std::fmt::Debug for DefaultLineIterator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator")
            .field("text", &self.text)
            .finish()
    }
}

impl<'a> DefaultLineIterator<'a> {
    pub(crate) fn new(
        text: &'a str,
        max_width: u32,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Self {
        let tokens = tokens.peekable();
        Self {
            max_width,
            text,
            tokens,
        }
    }
}

impl<'a> Iterator for DefaultLineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.tokens.peek()?.clone();
        let mut last = first.clone();

        // build a line by collecting tokens up to max_width,
        // or if a newline is found
        let width = 0;

        while let Some(token) = self.tokens.peek() {
            todo!();
            /*
            let next_width = (token.span.end - start) as u32;

            // taking another token would make the line too long
            if next_width > self.max_width {
                break;
            }

            // Width is fine, pull the token
            let token = self.tokens.next().unwrap();

            // newlines cause the current line to stop
            if token.kind == TokenKind::Newline {
                break;
            }

            last = token;
            */
        }

        Some(&self.text[first.range.start..last.range.end])
    }
}
