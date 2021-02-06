use std::{fmt::Formatter, iter::Peekable};

use crate::tokenize::Token;

pub trait LineBuilder: std::fmt::Debug {
    fn build<'a>(
        &self,
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
        text: &'a str,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let tokens = tokens.peekable();
        Box::new(DefaultLineIterator { text, tokens })
    }
}

/// Provides lines as `&str`
pub struct DefaultLineIterator<'a> {
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
    pub(crate) fn new(text: &'a str, tokens: Box<dyn Iterator<Item = Token> + 'a>) -> Self {
        let tokens = tokens.peekable();
        Self { text, tokens }
    }
}

impl<'a> Iterator for DefaultLineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.tokens.peek()?.clone();
        let mut last = first.clone();

        Some(&self.text[first.span.start..last.span.end])
    }
}
