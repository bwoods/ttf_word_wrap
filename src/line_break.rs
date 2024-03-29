use std::fmt::Formatter;

use crate::{
    partial_tokens::{PartialToken, PartialTokens},
    token::{Kind, TokenKind},
};

/// A trait for injecting Synthetic newlines at a display_width.
pub trait AddNewlines<T> {
    fn add_newlines_at(self, max_width: u32) -> LineBreakIterator<T>;
}

impl<T> AddNewlines<T> for T
where
    T: PartialTokens<Item = PartialToken>,
{
    fn add_newlines_at(self, max_width: u32) -> LineBreakIterator<T> {
        LineBreakIterator {
            max_width,
            tokens: self,
            width_remaining: max_width,
            previous_token_kind: None,
            force_newline: false,
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

    force_newline: bool,
}

impl<T> LineBreakIterator<T>
where
    T: PartialTokens<Item = PartialToken>,
{
    fn newline(&mut self) {
        self.width_remaining = self.max_width;
        self.previous_token_kind.take();
    }
}

impl<T> std::fmt::Debug for LineBreakIterator<T>
where
    T: std::fmt::Debug,
    T: PartialTokens<Item = PartialToken>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenWrapIterator")
            .field("max_width", &self.max_width)
            .finish()
    }
}

impl<T> Iterator for LineBreakIterator<T>
where
    T: PartialTokens<Item = PartialToken>,
{
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        if self.force_newline {
            self.force_newline = false;
            return Some(TokenKind::Newline(None));
        }

        // Take tokens until there is no more space left (or the next token doesn't fit)
        while let Some(partial_token) = self.tokens.next(self.width_remaining) {
            return match partial_token {
                PartialToken::TokenOverflow(token_kind) => {
                    // Only force the newline if there is a token following this one
                    if self.tokens.peek(self.max_width).is_some() {
                        self.force_newline = true;
                    }
                    Some(token_kind)
                }
                PartialToken::Token(token_kind) => {
                    // Skip optional tokens at the beginning of a line
                    if self.previous_token_kind.is_none() && token_kind.is_optional() {
                        continue;
                    }

                    let token_kind = match token_kind {
                        TokenKind::Required(token) => TokenKind::Required(token),
                        TokenKind::Optional(token) => {
                            // Only return the Optional token_kind if a Required one fits on the line after it
                            let width_remaining = self.width_remaining - token.display_width;
                            match self.tokens.peek(width_remaining) {
                                Some(PartialToken::Token(_)) => TokenKind::Optional(token),
                                None
                                | Some(PartialToken::TokenOverflow(_))
                                | Some(PartialToken::EndOfLine) => {
                                    // The following token will not fit on the line, do not keep the current
                                    // optional token, replace it with a synthetic newline
                                    TokenKind::Newline(None)
                                }
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

                    Some(token_kind)
                }
                PartialToken::EndOfLine => {
                    self.newline();
                    Some(TokenKind::Newline(None))
                }
            };
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        grapheme_width::WithGraphemeWidth, partial_tokens::WithPartialTokens, token::Token,
        whitespace::TokenizeWhiteSpace, TTFParserMeasure,
    };

    use super::*;

    #[test]
    fn terminates() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "";
        let mut tokens = text
            .with_grapheme_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(5000, text, &measure)
            .add_newlines_at(5000);

        assert!(tokens.next().is_none());
    }

    #[test]
    fn consecutive_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\n\n\r\n";
        let mut tokens = text
            .with_grapheme_width(&measure)
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
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\n1234";
        let mut tokens = text
            .with_grapheme_width(&measure)
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
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let mut tokens = text
            .with_grapheme_width(&measure)
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
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "123\n456\r\n7890";
        let mut tokens = text
            .with_grapheme_width(&measure)
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
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "this is a test";
        let mut tokens = text
            .with_grapheme_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20_000, text, &measure)
            .add_newlines_at(20_000);

        // Optional space token_kinds should be returned
        ["this", " ", "is", " ", "a", " ", "test"]
            .iter()
            .for_each(|&expected| {
                let token = tokens.next().unwrap().into_token().unwrap();
                let text = token.as_str(text);
                assert_eq!(text, expected);
            });

        assert!(tokens.next().is_none());
    }

    #[test]
    fn no_width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "TEST";
        let mut tokens = text
            .with_grapheme_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(0, text, &measure)
            .add_newlines_at(0);

        let token = tokens.next().unwrap();
        assert!(matches!(
            token,
            TokenKind::Required(Token {
                start: 0,
                end: 1,
                ..
            })
        ));

        let token = tokens.next().unwrap();
        assert!(matches!(token, TokenKind::Newline(None)));

        let token = tokens.next().unwrap();
        assert!(matches!(
            token,
            TokenKind::Required(Token {
                start: 1,
                end: 2,
                ..
            })
        ));

        let token = tokens.next().unwrap();
        assert!(matches!(token, TokenKind::Newline(None)));

        let token = tokens.next().unwrap();
        assert!(matches!(
            token,
            TokenKind::Required(Token {
                start: 2,
                end: 3,
                ..
            })
        ));
        let token = tokens.next().unwrap();
        assert!(matches!(token, TokenKind::Newline(None)));

        let token = tokens.next().unwrap();
        assert!(matches!(
            token,
            TokenKind::Required(Token {
                start: 3,
                end: 4,
                ..
            })
        ));

        assert!(tokens.next().is_none());
    }
}
