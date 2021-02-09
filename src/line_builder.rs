use std::fmt::Formatter;

use crate::{partial_tokens::PartialTokens, Token};

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
        let mut start_token = None;
        let mut end_token = None;

        dbg!();

        while let Some(token) = dbg!(self.tokens.next(width_remaining)) {
            let token_width = token.width;
            dbg!(width_remaining, token_width);
            if start_token.is_none() {
                start_token.replace(token);
            } else {
                end_token.replace(token);
            }
            width_remaining -= token_width;
        }

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
        tokenize::whitespace::TokenizeWhiteSpace,
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

        // // no more
        // let token = lines.next();
        // assert!(token.is_none());
    }
}
