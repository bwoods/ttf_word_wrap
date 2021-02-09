use crate::tokenize::Token;

pub trait WithPartialTokens<T> {
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<T>;
}

impl<T> WithPartialTokens<T> for T
where
    T: Iterator<Item = Token>,
{
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<T> {
        PartialTokensIterator {
            max_width,
            tokens: self,
            partial: None,
        }
    }
}

pub trait PartialTokens {
    type Item;

    fn next(&mut self, space_remaining: usize) -> Option<Self::Item>;
}

pub struct PartialTokensIterator<T> {
    max_width: u32,
    tokens: T,
    partial: Option<Token>,
}

impl<T> PartialTokensIterator<T> {
    pub fn new(tokens: T, max_width: u32) -> Self {
        Self {
            tokens,
            partial: None,
            max_width,
        }
    }

    fn process_partial(&mut self, token: Token, space_remaining: usize) -> Option<Token> {
        if token.width > self.max_width {
            let (head, tail) = token.split_at(space_remaining);
            self.partial.replace(tail);
            Some(head)
        } else {
            Some(token)
        }
    }
}

impl<T> PartialTokens for PartialTokensIterator<T>
where
    T: Iterator<Item = Token>,
{
    type Item = Token;

    fn next(&mut self, space_remaining: usize) -> Option<Self::Item> {
        match self.partial.take() {
            Some(partial) => self.process_partial(partial, space_remaining),
            None => {
                let token = self.tokens.next()?;
                self.process_partial(token, space_remaining)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth,
        tokenize::{whitespace::TokenizeWhiteSpace, TokenKind},
    };

    use super::*;

    #[test]
    fn tiny_width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let mut partials = "aoeuaoeuaoeuaoeaoeu"
            .with_char_width(font_face)
            .tokenize_white_space()
            .with_partial_tokens(1000);

        let token = partials.next(1000).unwrap();
        assert!(matches!(
            Token {
                kind: TokenKind::Required,
                start: 0,
                end: 1,
                width: 2500
            },
            token
        ));
    }
}
