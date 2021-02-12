use std::{fmt::Formatter, str::Chars};

use crate::{token::TokenKind, Measure};

/// A `char`s position in lines of text
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Position {
    /// The `char` for the font glyph
    pub ch: char,

    /// The line that this char is on
    pub line: u32,

    /// The horizontal offset in the same units as the Font
    pub offset: u32,
}

pub trait Positions<T> {
    fn positions<'a>(self, text: &'a str, measure: &'a dyn Measure) -> PositionIterator<'a, T>;
}

impl<T> Positions<T> for T
where
    T: Iterator<Item = TokenKind>,
{
    fn positions<'a>(self, text: &'a str, measure: &'a dyn Measure) -> PositionIterator<'a, T> {
        PositionIterator {
            chars: None,
            display_offset: 0,
            line: 0,
            measure,
            text,
            tokens: self,
        }
    }
}

/// Provides lines as `&str`
#[derive(Clone)]
pub struct PositionIterator<'a, T> {
    chars: Option<Chars<'a>>,
    display_offset: u32,
    line: u32,
    measure: &'a dyn Measure,
    text: &'a str,
    tokens: T,
}

impl<'a, T> std::fmt::Debug for PositionIterator<'a, T>
where
    T: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PositionIterator").finish()
    }
}

impl<'a, T> Iterator for PositionIterator<'a, T>
where
    T: Iterator<Item = TokenKind>,
{
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        match self.chars.as_mut() {
            Some(chars) => match chars.next() {
                Some(ch) => {
                    // There is a char! Measure it and create the Position
                    let offset = self.display_offset;
                    // add this glyph's width to the display_offset
                    self.display_offset += u32::from(self.measure.char(ch));
                    Position {
                        ch,
                        line: self.line,
                        offset,
                    }
                    .into()
                }
                None => {
                    // there are no more chars, set it to None and retry
                    self.chars.take();
                    self.next()
                }
            },
            None => match self.tokens.next() {
                Some(TokenKind::Newline(_)) => {
                    // Increment the line and call again
                    self.line += 1;
                    self.display_offset = 0;
                    self.next()
                }
                Some(TokenKind::Optional(token)) | Some(TokenKind::Required(token)) => {
                    // There is another token, prep chars
                    let chars = self.text[token.start..token.end].chars();
                    self.chars.replace(chars);
                    self.next()
                }
                None => {
                    // End the iteration, no more tokens
                    return None;
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::{
        char_width::WithCharWidth, line_break::AddNewlines, partial_tokens::WithPartialTokens,
        whitespace::TokenizeWhiteSpace, TTFParserMeasure,
    };

    use super::*;

    #[test]
    fn one_glyph() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "A";
        let mut positions = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20_000, text, &measure)
            .add_newlines_at(20_000)
            .positions(text, &measure);

        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'A',
                line: 0,
                offset: 0
            },
            token
        );

        assert!(positions.next().is_none());
    }

    #[test]
    fn newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "AB\nCD";
        let mut positions = text
            .with_char_width(&measure)
            .tokenize_white_space(&measure)
            .with_partial_tokens(20_000, text, &measure)
            .add_newlines_at(20_000)
            .positions(text, &measure);

        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'A',
                line: 0,
                offset: 0
            },
            token
        );
        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'B',
                line: 0,
                offset: 1336
            },
            token
        );
        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'C',
                line: 1,
                offset: 0
            },
            token
        );
        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'D',
                line: 1,
                offset: 1333
            },
            token
        );

        assert!(positions.next().is_none());
    }
}
