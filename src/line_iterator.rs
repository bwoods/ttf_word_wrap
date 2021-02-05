use std::vec::IntoIter;

use crate::tokenize::span::Span;

/// Provides lines as `&str`
#[derive(Debug)]
pub struct LineIterator<'a> {
    text: &'a str,
    spans: IntoIter<Span>,
}

impl<'a> LineIterator<'a> {
    pub(crate) fn new(text: &'a str, spans: Vec<Span>) -> Self {
        let spans = spans.into_iter();
        Self { text, spans }
    }
}

impl<'a> Iterator for LineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let span = self.spans.next()?;
        Some(&self.text[span.start..span.end])
    }
}
