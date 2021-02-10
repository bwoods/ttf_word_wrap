use std::fmt::Formatter;

use crate::{
    partial_tokens::PartialTokens,
    token::{Token, TokenKind},
};

pub trait Lines<T> {
    fn lines(self, max_width: u32) -> LineIterator<T>;
}

impl<'a, T> Lines<T> for T
where
    T: PartialTokens<Item = Token<'a>> + 'a,
{
    fn lines(self, max_width: u32) -> LineIterator<T> {
        LineIterator {
            max_width,
            tokens: self,
        }
    }
}

/// Provides lines as `&str`
#[derive(Clone, PartialEq)]
pub struct LineIterator<T> {
    max_width: u32,
    tokens: T,
}

impl<T> std::fmt::Debug for LineIterator<T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator")
            .field("max_width", &self.max_width)
            .finish()
    }
}

impl<'a, T> Iterator for LineIterator<T>
where
    T: PartialTokens<Item = Token<'a>> + 'a,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut width_remaining = self.max_width;
        let mut start_token: Option<Token<'a>> = None;
        let mut last_token: Option<Token<'a>> = None;
        let mut next_to_last_token: Option<Token<'a>> = None;

        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(token) = self.tokens.next(width_remaining) {
            let token_width = token.width;

            // Skip optional tokens at the beginning of a line
            if start_token.is_none() && token.kind == TokenKind::Optional {
                continue;
            }

            if start_token.is_none() {
                // Keep track of the first token
                start_token.replace(token);
            } else {
                // So that the last token can be discarded later, track the next-to-last token
                if let Some(end_token) = last_token.take() {
                    next_to_last_token.replace(end_token);
                }
                last_token.replace(token);
            }

            width_remaining -= token_width;
        }

        // Swap the previous and the end tokens if the end token is optional
        let end_token_kind = last_token.as_ref().map(|t| t.kind.clone());

        let end_token = match (next_to_last_token, last_token, end_token_kind) {
            (Some(previous), _, Some(TokenKind::Optional)) => Some(previous),
            (_, end, _) => end,
        };

        match (start_token, end_token) {
            (None, None) => None,
            (None, Some(_)) => unreachable!(),
            (Some(token), None) => Some(&token.text[token.start..token.end]),
            (Some(start_token), Some(end_token)) => {
                Some(&start_token.text[start_token.start..end_token.end])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, partial_tokens::WithPartialTokens,
        whitespace::TokenizeWhiteSpace,
    };

    use super::*;

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let text = "1234567890";
        let mut lines = text
            .with_char_width(&font_face)
            .tokenize_white_space()
            .with_partial_tokens(5000)
            .lines(5000);

        let token = lines.next().unwrap();
        assert_eq!("1234", token);

        let token = lines.next().unwrap();
        assert_eq!("5678", token);

        let token = lines.next().unwrap();
        assert_eq!("90", token);
    }
}
