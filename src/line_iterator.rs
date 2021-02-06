use std::{fmt::Formatter, iter::Peekable};

use crate::{tokenize::Token, Options};

trait LineBuilder {}

/// Provides lines as `&str`
pub struct LineIterator<'a> {
    text: &'a str,
    tokens: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

impl<'a> std::fmt::Debug for LineIterator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator")
            .field("text", &self.text)
            .finish()
    }
}

impl<'a> LineIterator<'a> {
    pub(crate) fn new(
        text: &'a str,
        options: Options,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Self {
        let tokens = tokens.peekable();
        Self { text, tokens }
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.tokens.peek()?.clone();
        let mut last = first.clone();

        Some(&self.text[first.span.start..last.span.end])
    }
}
