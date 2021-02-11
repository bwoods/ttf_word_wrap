use std::fmt::Formatter;

use crate::{partial_tokens::PartialTokens, token::TokenKind};

/// Provides lines as `&str`
#[derive(Clone)]
pub struct LineWidthNewlineIterator<'a, T>
where
    T: PartialTokens<Item = TokenKind<'a>> + 'a,
{
    /// Maximum display width for the lines
    max_width: u32,

    /// Tokens used to fill the line
    tokens: T,

    /// The space remaining in the line we are filling
    width_remaining: u32,

    /// We need the previous token to create TokenKind::Newline
    /// `previous_token` is `None` if the iterator is at the beginning of a line.
    previous_token: Option<TokenKind<'a>>,
}

impl<'a, T> LineWidthNewlineIterator<'a, T>
where
    T: PartialTokens<Item = TokenKind<'a>> + 'a,
{
    pub fn newline(&mut self) {
        self.width_remaining = self.max_width;
        self.previous_token.take();
    }
}

impl<'a, T> std::fmt::Debug for LineWidthNewlineIterator<'a, T>
where
    T: std::fmt::Debug,
    T: PartialTokens<Item = TokenKind<'a>> + 'a,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenWrapIterator")
            .field("max_width", &self.max_width)
            .finish()
    }
}

impl<'a, T> Iterator for LineWidthNewlineIterator<'a, T>
where
    T: PartialTokens<Item = TokenKind<'a>> + 'a,
{
    type Item = TokenKind<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // if next is newline, return it and reset()
        // if no previous and next is optional, skip it
        //
        // if next is optional, check if peek() fits, if not, return newline.
        //
        //
        //
        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(token_kind) = self.tokens.next(self.width_remaining) {
            // newlines pass through
            if token_kind.is_newline() {
                self.newline();
                return Some(token_kind);
            }

            // Skip optional tokens at the beginning of a line
            if self.previous_token.is_some() && token_kind.is_optional() {
                continue;
            }

            let token_width = token_kind.width();

            // Only return the Optional token_kind if a Required one fits on the line after it
            if token_kind.is_optional() {
                let width_remaining = self.width_remaining - token_width;
                let peek = self.tokens.peek(token_width);
                //if peek
            }

            // token accepted, no longer at the start of a line
            unimplemented!(); // self.at_line_start = false;
            self.width_remaining -= token_width;
        }

        // No more space in the line, reset and return a Newline token
        self.newline();

        unimplemented!()
    }
}
