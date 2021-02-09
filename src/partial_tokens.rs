use crate::Token;

pub trait WithPartialTokens<'a, T> {
    fn with_partial_tokens(self, max_width: u32) -> PartialTokensIterator<'a, T>;
}

impl<'a, T> WithPartialTokens<'a, T> for T
where
    T: Iterator<Item = Token<'a>> + 'a,
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

pub struct PartialTokensIterator<'a, T> {
    max_width: u32,
    tokens: T,
    partial: Option<Token<'a>>,
}

impl<'a, T> PartialTokensIterator<'a, T> {
    fn process_partial(&mut self, token: Token<'a>, space_remaining: u32) -> Option<Token<'a>> {
        if token.width > self.max_width {
            // If the word is wider than the max_width we break it anywhere
            match token.split_at(space_remaining) {
                (None, None) => None,
                (None, Some(partial)) => {
                    self.partial.replace(partial);
                    None
                }
                (Some(partial), None) => Some(partial),
                (Some(head), Some(tail)) => {
                    self.partial.replace(tail);
                    Some(head)
                }
            }
        } else if token.width > space_remaining {
            // If the word is not wider than the max width and the line doesn't have room, return
            // None
            self.partial.replace(token);
            None
        } else {
            // There is room on the line for the token
            Some(token)
        }
    }
}

impl<'a, T> PartialTokens for PartialTokensIterator<'a, T>
where
    T: Iterator<Item = Token<'a>> + 'a,
{
    type Item = Token<'a>;

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

    use crate::{char_width::WithCharWidth, tokenize::whitespace::TokenizeWhiteSpace};

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
        let token = partials.next(3000).unwrap();
        assert_eq!("ao", token.to_string());

        // a full line width
        let token = partials.next(10000).unwrap();
        assert_eq!("euaoeuao", token.to_string());

        // not enough room for a character
        let token = partials.next(500);
        assert!(token.is_none());
    }
}
