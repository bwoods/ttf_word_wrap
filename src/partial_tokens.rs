use std::iter::Peekable;

use crate::token::TokenKind;

pub trait WithPartialTokens<'a, T>
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<'a, T>;
}

impl<'a, T> WithPartialTokens<'a, T> for T
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<'a, T> {
        PartialTokensIterator {
            max_width,
            tokens: self.peekable(),
            partial: None,
        }
    }
}

pub trait PartialTokens {
    type Item;

    fn next(&mut self, space_remaining: u32) -> Option<Self::Item>;
    fn peek(&mut self, space_remaining: u32) -> Option<Self::Item>;
}

#[derive(Clone, Debug)]
pub struct PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    max_width: u32,
    tokens: Peekable<T>,
    partial: Option<TokenKind<'a>>,
}

// Takes a token and space remaining, returns a tuple of the head and tail of the split token.
fn process_partial<'a>(
    max_width: u32,
    token_kind: TokenKind<'a>,
    space_remaining: u32,
) -> (Option<TokenKind<'a>>, Option<TokenKind<'a>>) {
    let kind = token_kind.kind();

    // Newlines pass though
    if token_kind.is_newline() {
        return (Some(token_kind), None);
    }

    let token = token_kind.into_token();

    if token.width > max_width {
        // If the word is wider than the max_width we break it anywhere
        let (head, tail) = token.split_at(space_remaining);
        let head = head.map(|t| kind.token(t));
        let tail = tail.map(|t| kind.token(t));
        (head, tail)
    } else if token.width > space_remaining {
        // If the word is not wider than the max width and the line doesn't have room, return
        // None
        // self.partial.replace(kind.token(token));
        (None, Some(kind.token(token)))
    } else {
        // There is room on the line for the token
        (Some(kind.token(token)), None)
    }
}

impl<'a, T> PartialTokens for PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    type Item = TokenKind<'a>;

    fn next(&mut self, space_remaining: u32) -> Option<Self::Item> {
        let head_tail = match self.partial.take() {
            Some(partial) => process_partial(self.max_width, partial, space_remaining),
            None => {
                let token = self.tokens.next()?;
                process_partial(self.max_width, token, space_remaining)
            }
        };

        // If there is a tail, preserve it, return the heads
        match head_tail {
            (head, None) => head,
            (head, Some(tail)) => {
                self.partial.replace(tail);
                head
            }
        }
    }

    fn peek(&mut self, space_remaining: u32) -> Option<Self::Item> {
        let head_tail = match self.partial.clone() {
            Some(partial) => process_partial(self.max_width, partial, space_remaining),
            None => {
                let token = self.tokens.peek()?.clone();
                process_partial(self.max_width, token, space_remaining)
            }
        };

        // If there is a tail, preserve it, return the heads
        match head_tail {
            (head, _) => head,
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, display_width::TTFParserDisplayWidth,
        whitespace::TokenizeWhiteSpace,
    };

    use super::*;

    #[test]
    fn partials() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "aoeuaoeuaoeuaoeaoeu";
        let mut partials = text
            .with_char_width(&display_width)
            .tokenize_white_space(&display_width)
            .with_partial_tokens(10000);

        // get a few tokens
        let token = partials.next(3000).unwrap().into_token();
        assert_eq!("ao", token.to_string());

        // a full line width
        let token = partials.next(10000).unwrap().into_token();
        assert_eq!("euaoeuao", token.to_string());

        // not enough room for a character
        let token = partials.next(500);
        assert!(token.is_none());
    }

    #[test]
    fn partial_newline() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let text = "aoeu\naoeu";
        let mut partials = text
            .with_char_width(&display_width)
            .tokenize_white_space(&display_width)
            .with_partial_tokens(20000);

        //
        let token = partials.next(10000).unwrap().into_token();
        assert_eq!("aoeu", token.to_string());

        // get a few tokens
        let _token_kind = partials.next(10000).unwrap();
        assert!(matches!(TokenKind::Newline, _token_kind));

        // a full line width
        let token = partials.next(10000).unwrap().into_token();
        assert_eq!("aoeu", token.to_string());
    }
}
