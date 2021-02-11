use std::fmt::Formatter;

use crate::{
    partial_tokens::PartialTokens,
    token::{Kind, TokenKind},
};

/// A trait for injecting Synthetic newlines at a display_width.
pub trait AddNewlines<T> {
    fn add_newlines_at(self, max_width: u32) -> LineBreakIterator<T>;
}

impl<T> AddNewlines<T> for T
where
    T: PartialTokens<Item = TokenKind>,
{
    fn add_newlines_at(self, max_width: u32) -> LineBreakIterator<T> {
        LineBreakIterator {
            max_width,
            tokens: self,
            width_remaining: max_width,
            previous_token_kind: None,
        }
    }
}

/// Injects Synthetic newlines into the token stream at a given display width.
#[derive(Clone)]
pub struct LineBreakIterator<T> {
    /// Maximum display width for the lines
    max_width: u32,

    /// Tokens used to fill the line
    tokens: T,

    /// The space remaining in the line we are filling
    width_remaining: u32,

    /// We need the previous token to create TokenKind::Newline
    /// `previous_token` is `None` if the iterator is at the beginning of a line.
    previous_token_kind: Option<Kind>,
}

impl<T> LineBreakIterator<T>
where
    T: PartialTokens<Item = TokenKind>,
{
    fn newline(&mut self) {
        self.width_remaining = self.max_width;
        self.previous_token_kind.take();
    }

    fn is_finished(&mut self) -> bool {
        self.tokens.peek(self.max_width).is_none()
    }
}

impl<T> std::fmt::Debug for LineBreakIterator<T>
where
    T: std::fmt::Debug,
    T: PartialTokens<Item = TokenKind>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenWrapIterator")
            .field("max_width", &self.max_width)
            .finish()
    }
}

impl<T> Iterator for LineBreakIterator<T>
where
    T: PartialTokens<Item = TokenKind>,
{
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(token_kind) = self.tokens.next(self.width_remaining) {
            // Skip optional tokens at the beginning of a line
            if self.previous_token_kind.is_none() && token_kind.is_optional() {
                continue;
            }

            let token_kind = match token_kind {
                TokenKind::Required(token) => TokenKind::Required(token),
                TokenKind::Optional(token) => {
                    // Only return the Optional token_kind if a Required one fits on the line after it
                    let width_remaining = self.width_remaining - token.display_width;
                    if self.tokens.peek(width_remaining).is_some() {
                        TokenKind::Optional(token)
                    } else {
                        // The following token will not fit on the line, do not keep the current
                        // optional token, replace it with a synthetic newline
                        TokenKind::Newline(None)
                    }
                }
                TokenKind::Newline(token) => {
                    // All types of newlines pass through
                    TokenKind::Newline(token)
                }
            };

            if token_kind.is_newline() {
                self.newline();
            } else {
                // token accepted, no longer at the start of a line
                self.width_remaining -= token_kind.width();
                self.previous_token_kind.replace(token_kind.kind());
            }
            return Some(token_kind);
        }

        if self.is_finished() {
            None
        } else {
            self.newline();
            Some(TokenKind::Newline(None))
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, partial_tokens::WithPartialTokens,
        whitespace::TokenizeWhiteSpace, TTFParserMeasure,
    };

    use super::*;

    #[test]
    fn terminates() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000);

        assert!(tokens.next().is_none());
    }

    #[test]
    fn consecutive_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\n\n\r\n";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000);

        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));
        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));
        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));
        assert!(tokens.next().is_none());
    }

    #[test]
    fn starting_newline() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\n1234";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000);

        // not a synthetic newline
        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "1234");

        assert!(tokens.next().is_none());
    }

    #[test]
    fn synthetic_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(6000, text, &measure)
            .add_newlines_at(6000);

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "12345");

        // Synthetic Newline
        assert!(matches!(tokens.next(), Some(TokenKind::Newline(None))));

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "67890");

        assert!(tokens.next().is_none());
    }

    #[test]
    fn with_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "123\n456\r\n7890";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000);

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "123");

        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "456");

        assert!(matches!(tokens.next(), Some(TokenKind::Newline(Some(_)))));

        let token = tokens.next().unwrap().into_token().unwrap();
        assert_eq!(token.as_str(text), "7890");

        assert!(tokens.next().is_none());
    }

    #[test]
    fn optional_tokens() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "this is a test";
        let mut tokens = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20_000, text, &measure)
            .add_newlines_at(20_000);

        // Optional space token_kinds should be returned
        ["this", " ", "is", " ", "a", " ", "test"]
            .iter()
            .for_each(|&expected| {
                let token = tokens.next().unwrap().into_token().unwrap();
                assert_eq!(token.as_str(text), expected);
            });

        assert!(tokens.next().is_none());
    }
}
