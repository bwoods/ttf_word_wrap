use std::iter::Peekable;

use crate::{
    token::{Token, TokenKind},
    Measure,
};

pub trait WithPartialTokens<T>
where
    T: Iterator<Item = TokenKind>,
{
    fn with_partial_tokens<'a>(
        self,
        max_width: u32,
        text: &'a str,
        measure: &'a dyn Measure,
    ) -> PartialTokensIterator<'a, T>;
}

impl<T> WithPartialTokens<T> for T
where
    T: Iterator<Item = TokenKind>,
{
    fn with_partial_tokens<'a>(
        self,
        max_width: u32,
        text: &'a str,
        measure: &'a dyn Measure,
    ) -> PartialTokensIterator<'a, T> {
        PartialTokensIterator {
            text,
            measure,
            max_width,
            tokens: self.peekable(),
            partial: None,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PartialToken {
    Token(TokenKind),
    EndOfLine,
}

impl PartialToken {
    pub fn into_token(self) -> Option<Token> {
        match self {
            PartialToken::Token(token_kind) => token_kind.into_token(),
            PartialToken::EndOfLine => None,
        }
    }
    pub fn into_tokenkind(self) -> Option<TokenKind> {
        match self {
            PartialToken::Token(token_kind) => Some(token_kind),
            PartialToken::EndOfLine => None,
        }
    }
}

pub trait PartialTokens {
    type Item;

    fn next(&mut self, space_remaining: u32) -> Option<Self::Item>;
    fn peek(&mut self, space_remaining: u32) -> Option<Self::Item>;
}

/// Splits tokens that are wider than the max_width
#[derive(Clone, Debug)]
pub struct PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind>,
{
    text: &'a str,
    measure: &'a dyn Measure,
    max_width: u32,
    tokens: Peekable<T>,
    partial: Option<TokenKind>,
}

// Takes a token and space remaining, returns a tuple of the head and tail of the split token.
impl<'a, T> PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind>,
{
    fn process_partial(
        &self,
        token_kind: TokenKind,
        space_remaining: u32,
    ) -> (Option<TokenKind>, Option<TokenKind>) {
        let kind = token_kind.kind();

        let token = match token_kind {
            TokenKind::Required(token) | TokenKind::Optional(token) => token,
            newline @ TokenKind::Newline(_) => {
                // Newlines pass though
                return (Some(newline), None);
            }
        };

        if token.display_width > self.max_width {
            // If the word is wider than the max_width we break it anywhere
            let (head, tail) = token.split_at(space_remaining, self.text, self.measure);
            let head = head.map(|t| kind.token(t));
            let tail = tail.map(|t| kind.token(t));
            (head, tail)
        } else if token.display_width > space_remaining {
            // If the word is not wider than the max width and the line doesn't have room, return
            // None
            // self.partial.replace(kind.token(token));
            (None, Some(kind.token(token)))
        } else {
            // There is room on the line for the token
            (Some(kind.token(token)), None)
        }
    }
}

impl<'a, T> PartialTokens for PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind>,
{
    type Item = PartialToken;

    fn next(&mut self, space_remaining: u32) -> Option<PartialToken> {
        let head_tail = match self.partial.take() {
            Some(partial) => self.process_partial(partial, space_remaining),
            None => {
                let token = self.tokens.next()?;
                self.process_partial(token, space_remaining)
            }
        };

        // If there is a tail, preserve it, return the heads
        match head_tail {
            (head, None) => head.map(PartialToken::Token),
            (head, Some(tail)) => {
                self.partial.replace(tail);
                head.map_or_else(
                    || Some(PartialToken::EndOfLine),
                    |t| Some(PartialToken::Token(t)),
                )
            }
        }
    }

    fn peek(&mut self, space_remaining: u32) -> Option<PartialToken> {
        let head_tail = match self.partial.clone() {
            Some(partial) => self.process_partial(partial, space_remaining),
            None => {
                let token = self.tokens.peek()?.clone();
                self.process_partial(token, space_remaining)
            }
        };

        match head_tail {
            (head, None) => head.map(PartialToken::Token),
            (head, Some(_)) => head.map_or_else(
                || Some(PartialToken::EndOfLine),
                |t| Some(PartialToken::Token(t)),
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, measure::TTFParserMeasure, whitespace::TokenizeWhiteSpace,
    };

    use super::*;

    #[test]
    fn partials() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "aoeuaoeuaoeuaoeaoeu";
        let mut partials = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(10000, text, &measure);

        // get a few tokens
        let token = partials.next(3000).unwrap().into_token().unwrap();
        assert_eq!("ao", token.as_str(text));

        // a full line width
        let token = partials.next(10000).unwrap().into_token().unwrap();
        assert_eq!("euaoeuao", token.as_str(text));

        // not enough room for a character
        let token = partials.next(500).unwrap();
        assert!(matches!(token, PartialToken::EndOfLine));

        let token = partials.next(10000).unwrap().into_token().unwrap();
        assert_eq!("euaoeaoe", token.as_str(text));

        let token = partials.next(10000).unwrap().into_token().unwrap();
        assert_eq!("u", token.as_str(text));

        let token = partials.next(500);
        assert!(token.is_none());
    }

    #[test]
    fn partial_newline() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "aoeu\naoeu";
        let mut partials = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20000, text, &measure);

        //
        let token = partials.next(10000).unwrap().into_token().unwrap();
        assert_eq!("aoeu", token.as_str(text));

        // get a few tokens
        let _token_kind = partials.next(10000).unwrap();
        assert!(matches!(TokenKind::Newline, _token_kind));

        // a full line width
        let token = partials.next(10000).unwrap().into_token().unwrap();
        assert_eq!("aoeu", token.as_str(text));
    }

    #[test]
    fn peek_next() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\naoeu";
        let mut partials = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20000, text, &measure);

        let token = partials.peek(3000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());

        let token = partials.next(20000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());

        let token = partials.peek(20000).unwrap().into_token().unwrap();
        assert_eq!("aoeu", token.as_str(text));

        let token = partials.next(20000).unwrap().into_token().unwrap();
        assert_eq!("aoeu", token.as_str(text));

        let token = partials.peek(15000);
        assert!(token.is_none());

        let token = partials.next(15000);
        assert!(token.is_none());
    }

    #[test]
    fn peek_peek() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\naoeu";
        let mut partials = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20000, text, &measure);

        let token = partials.peek(3000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());

        let token = partials.peek(3000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());

        let token = partials.peek(3000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());

        let token = partials.peek(3000).unwrap().into_tokenkind().unwrap();
        assert!(token.is_newline());
    }
}
