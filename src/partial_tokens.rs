use crate::token::TokenKind;

pub trait WithPartialTokens<'a, T> {
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<'a, T>;
}

impl<'a, T> WithPartialTokens<'a, T> for T
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<'a, T> {
        PartialTokensIterator {
            max_width,
            tokens: self,
            partial: None,
        }
    }
}

pub trait PartialTokens {
    type Item;

    fn next(&mut self, space_remaining: u32) -> Option<Self::Item>;
}

#[derive(Clone, Debug)]
pub struct PartialTokensIterator<'a, T> {
    max_width: u32,
    tokens: T,
    partial: Option<TokenKind<'a>>,
}

impl<'a, T> PartialTokensIterator<'a, T> {
    fn process_partial(
        &mut self,
        token_kind: TokenKind<'a>,
        space_remaining: u32,
    ) -> Option<TokenKind<'a>> {
        let kind = token_kind.kind();

        // Newlines pass though
        if token_kind.is_newline() {
            return Some(token_kind);
        }

        let token = token_kind.into_token();

        if token.width > self.max_width {
            // If the word is wider than the max_width we break it anywhere
            match token.split_at(space_remaining) {
                (None, None) => None,
                (None, Some(partial)) => {
                    self.partial.replace(kind.token(partial));
                    None
                }
                (Some(partial), None) => Some(kind.token(partial)),
                (Some(head), Some(tail)) => {
                    self.partial.replace(kind.token(tail));
                    Some(kind.token(head))
                }
            }
        } else if token.width > space_remaining {
            // If the word is not wider than the max width and the line doesn't have room, return
            // None
            self.partial.replace(kind.token(token));
            None
        } else {
            // There is room on the line for the token
            Some(kind.token(token))
        }
    }
}

impl<'a, T> PartialTokens for PartialTokensIterator<'a, T>
where
    T: Iterator<Item = TokenKind<'a>> + 'a,
{
    type Item = TokenKind<'a>;

    fn next(&mut self, space_remaining: u32) -> Option<Self::Item> {
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

    use crate::{char_width::WithCharWidth, whitespace::TokenizeWhiteSpace};

    use super::*;

    #[test]
    fn partials() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let text = "aoeuaoeuaoeuaoeaoeu";
        let mut partials = text
            .with_char_width(&font_face)
            .tokenize_white_space()
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

        let text = "aoeu\naoeu";
        let mut partials = text
            .with_char_width(&font_face)
            .tokenize_white_space()
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
