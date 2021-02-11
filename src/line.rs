use std::fmt::Formatter;

use crate::{partial_tokens::PartialTokens, token::TokenKind};

pub trait Lines<T> {
    fn lines<'a>(self, text: &'a str, max_width: u32) -> LineIterator<'a, T>;
}

impl<T> Lines<T> for T
where
    T: PartialTokens<Item = TokenKind>,
{
    fn lines<'a>(self, text: &'a str, max_width: u32) -> LineIterator<'a, T> {
        LineIterator {
            text,
            max_width,
            tokens: self,
        }
    }
}

/// Provides lines as `&str`
#[derive(Clone, PartialEq)]
pub struct LineIterator<'a, T> {
    text: &'a str,
    max_width: u32,
    tokens: T,
}

impl<'a, T> std::fmt::Debug for LineIterator<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator")
            .field("max_width", &self.max_width)
            .finish()
    }
}

impl<'a, T> Iterator for LineIterator<'a, T>
where
    T: PartialTokens<Item = TokenKind> + 'a,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut width_remaining = self.max_width;
        let mut start_token: Option<TokenKind> = None;
        let mut last_token: Option<TokenKind> = None;
        let mut next_to_last_token: Option<TokenKind> = None;

        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(token_kind) = self.tokens.next(width_remaining) {
            let kind = token_kind.kind();

            if token_kind.is_newline() {
                break;
            }

            // Skip optional tokens at the beginning of a line
            if start_token.is_none() && token_kind.is_optional() {
                continue;
            }

            let token = token_kind.into_token();

            // record the width
            width_remaining -= token.display_width;

            if start_token.is_none() {
                // Keep track of the first token
                start_token.replace(kind.token(token));
            } else {
                // So that the last token can be discarded later, track the next-to-last token
                if let Some(end_token) = last_token.take() {
                    next_to_last_token.replace(end_token);
                }
                last_token.replace(kind.token(token));
            }
        }

        // Swap the previous and the end tokens if the end token is optional
        let end_token = match (next_to_last_token, last_token) {
            (Some(previous), Some(TokenKind::Optional(_))) => Some(previous),
            (_, last) => last,
        };

        match (
            start_token.map(TokenKind::into_token),
            end_token.map(TokenKind::into_token),
        ) {
            (None, None) => None,
            (None, Some(_)) => unreachable!(),
            (Some(token), None) => Some(&self.text[token.start..token.end]),
            (Some(start_token), Some(end_token)) => {
                Some(&self.text[start_token.start..end_token.end])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, display_width::TTFParserMeasure,
        partial_tokens::WithPartialTokens, whitespace::TokenizeWhiteSpace,
    };

    use super::*;

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let mut lines = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .lines(text, 5000);

        let token = lines.next().unwrap();
        assert_eq!("1234", token);

        let token = lines.next().unwrap();
        assert_eq!("5678", token);

        let token = lines.next().unwrap();
        assert_eq!("90", token);
    }

    #[test]
    fn with_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "123\n456\r\n7890";
        let mut lines = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .lines(text, 5000);

        let token = lines.next().unwrap();
        assert_eq!("123", token);

        let token = lines.next().unwrap();
        assert_eq!("456", token);

        let token = lines.next().unwrap();
        assert_eq!("7890", token);
    }
}
