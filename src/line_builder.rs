use std::{fmt::Formatter, iter::Peekable};

use crate::tokenize::{Token, TokenKind};

pub trait LineBuilder: std::fmt::Debug {
    fn build<'a>(
        &self,
        max_width: u32,
        tokens: Box<dyn Iterator<Item = Token>>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a>;
}

/// Provides lines as `&str`
#[derive(Debug)]
pub struct DefaultLineBuilder {}

impl DefaultLineBuilder {
    pub fn new() -> Self {
        Self {}
    }
}

impl LineBuilder for DefaultLineBuilder {
    fn build<'a, 'g>(
        &self,
        max_width: u32,
        tokens: Box<dyn Iterator<Item = Token> + 'a>,
    ) -> Box<dyn Iterator<Item = &'a str> + 'a> {
        let tokens = tokens.peekable();
        Box::new(DefaultLineIterator { max_width, tokens })
    }
}

/// Provides lines as `&str`
pub struct DefaultLineIterator<'a> {
    max_width: u32,
    tokens: Peekable<Box<dyn Iterator<Item = Token> + 'a>>,
}

impl<'a> std::fmt::Debug for DefaultLineIterator<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LineIterator").finish()
    }
}

impl<'a> Iterator for DefaultLineIterator<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let first = self.tokens.peek()?.clone();
        let mut last = first.clone();

        // build a line by collecting tokens up to max_width,
        // or if a newline is found
        let mut width = 0;

        while let Some(token_width) = self.tokens.peek() {
            let next_width = token_width.width + width;

            if next_width > self.max_width {
                // taking another token would make the line too long
                break;
            }

            // Width is fine, pull the token
            let token = self.tokens.next().unwrap();

            // newlines cause the current line to stop
            if token_width.token.kind == TokenKind::Newline {
                break;
            }

            width = next_width;
            last = token_width;
        }

        Some(&first.token.text[first.token.start..last.token.end])
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{tokenize::whitespace::TokenizeWhiteSpace, wrap_at::WrapAt};

    use super::*;

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let line_builder = DefaultLineBuilder::new();

        let tokens = "test".tokenize_white_space();

        let max_width = 3000;
        let max_width_tokens = Box::new(WrapAt::new(max_width, tokens));
        let lines: Vec<&str> = line_builder
            .build(max_width, max_width_tokens, &glyph_dimensions)
            .collect();

        // if the word is too long for the width given
        // | | width
        //  t est

        assert_eq!(lines, vec!["t", "e", "s", "t"]);
    }
}
