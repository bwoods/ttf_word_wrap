use std::fmt::Formatter;

use crate::token::TokenKind;

pub trait Lines<T> {
    fn lines<'a>(self, text: &'a str) -> LineIterator<'a, T>;
}

impl<T> Lines<T> for T
where
    T: Iterator<Item = TokenKind>,
{
    fn lines<'a>(self, text: &'a str) -> LineIterator<'a, T> {
        LineIterator { text, tokens: self }
    }
}

/// Provides lines as `&str`
#[derive(Clone, PartialEq)]
pub struct LineIterator<'a, T> {
    text: &'a str,
    tokens: T,
}

impl<'a, T> std::fmt::Debug for LineIterator<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator").finish()
    }
}

impl<'a, T> Iterator for LineIterator<'a, T>
where
    T: Iterator<Item = TokenKind>,
{
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let mut start_token: Option<TokenKind> = None;
        let mut last_token: Option<TokenKind> = None;

        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(token_kind) = self.tokens.next() {
            if start_token.is_none() {
                // Keep track of the first token
                start_token.replace(token_kind);
            } else if !token_kind.is_newline() {
                // last token should never be a synthetic newline because it does not have position
                // information
                last_token.replace(token_kind);
            }

            if token_kind.is_newline() {
                break;
            }
        }

        match (start_token, last_token) {
            // First token is newline, point to an empty str in the text
            (None, None) => None,
            // unreachable because the start must be filled before the last
            (None, Some(_)) => unreachable!(),
            // Only one token
            (Some(TokenKind::Newline(_)), None) => Some(&self.text[0..0]),
            (Some(TokenKind::Optional(token)), None) | (Some(TokenKind::Required(token)), None) => {
                Some(&self.text[token.start..token.end])
            }
            // Newlines have been stripped out
            (Some(TokenKind::Newline(_)), Some(_)) | (Some(_), Some(TokenKind::Newline(_))) => {
                unreachable!()
            }
            // Tokens are Optional or Required
            (Some(TokenKind::Optional(start_token)), Some(TokenKind::Optional(end_token)))
            | (Some(TokenKind::Required(start_token)), Some(TokenKind::Required(end_token)))
            | (Some(TokenKind::Optional(start_token)), Some(TokenKind::Required(end_token)))
            | (Some(TokenKind::Required(start_token)), Some(TokenKind::Optional(end_token))) => {
                Some(&self.text[start_token.start..end_token.end])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, display_width::TTFParserMeasure, line_break::AddNewlines,
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
            .add_newlines_at(5000)
            .lines(text);

        let token = lines.next().unwrap();
        assert_eq!("1234", token);

        let token = lines.next().unwrap();
        assert_eq!("5678", token);

        let token = lines.next().unwrap();
        assert_eq!("90", token);

        assert!(lines.next().is_none());
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
            .add_newlines_at(5000)
            .lines(text);

        let token = lines.next().unwrap();
        assert_eq!("123", token);

        let token = lines.next().unwrap();
        assert_eq!("456", token);

        let token = lines.next().unwrap();
        assert_eq!("7890", token);

        assert!(lines.next().is_none());
    }

    #[test]
    fn consecutive_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "123\n\r\n456";
        let mut lines = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000)
            .lines(text);

        // The first newline ends the first line
        let token = lines.next().unwrap();
        assert_eq!("123", token);

        // The second newline adds a line
        let token = lines.next().unwrap();
        assert_eq!("", token);

        let token = lines.next().unwrap();
        assert_eq!("456", token);

        assert!(lines.next().is_none());
    }

    #[test]
    fn starting_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\n1234";
        let mut lines = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000)
            .lines(text);

        let token = lines.next().unwrap();
        assert_eq!("", token);

        let token = lines.next().unwrap();
        assert_eq!("1234", token);

        assert!(lines.next().is_none());
    }

    #[test]
    fn word_wrap_small_words() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "caverns are not for the";
        let mut lines = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20_000, text, &measure)
            .add_newlines_at(20_000)
            .lines(text);

        let token = lines.next().unwrap();
        assert_eq!("caverns are not for", token);

        let token = lines.next().unwrap();
        assert_eq!("the", token);

        assert!(lines.next().is_none());
    }
}
