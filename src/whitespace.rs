use std::iter::Peekable;

use crate::{grapheme_width::GraphemeWidth, measure::Measure, token::Token, token::TokenKind};

#[derive(Copy, Clone, PartialEq)]
enum State {
    Newline,
    WhiteSpace,
    Other,
}

impl From<&str> for State {
    fn from(s: &str) -> Self {
        match s {
            "\r\n" | "\n" => State::Newline,
            s => {
                if let Some(first) = s.chars().next() {
                    if first.is_whitespace() {
                        return State::WhiteSpace;
                    }
                }

                State::Other
            }
        }
    }
}

pub trait TokenizeWhiteSpace<'a, T>
where
    T: Iterator<Item = GraphemeWidth<'a>> + Clone,
{
    fn tokenize_white_space(self, measure: &'a dyn Measure) -> WhiteSpaceIterator<'a, T>;
}

impl<'a, T> TokenizeWhiteSpace<'a, T> for T
where
    T: Iterator<Item = GraphemeWidth<'a>> + Clone,
{
    fn tokenize_white_space(self, measure: &'a dyn Measure) -> WhiteSpaceIterator<'a, T> {
        WhiteSpaceIterator::new(self.peekable(), measure)
    }
}

#[derive(Clone, Debug)]
pub struct WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = GraphemeWidth<'a>> + Clone,
{
    index: usize,
    grapheme_widths: Peekable<T>,
    measure: &'a dyn Measure,
}

impl<'a, T> WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = GraphemeWidth<'a>> + Clone,
{
    pub fn new(grapheme_widths: Peekable<T>, measure: &'a dyn Measure) -> Self {
        Self {
            grapheme_widths,
            index: 0,
            measure,
        }
    }
}

impl<'a, T> Iterator for WhiteSpaceIterator<'a, T>
where
    T: Iterator<Item = GraphemeWidth<'a>> + Clone,
{
    type Item = TokenKind;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the 'mode' for the next `Span` whitespace or not
        // will return None if there is no more text
        let grapheme_width = self.grapheme_widths.peek()?;
        let state = State::from(grapheme_width.grapheme);

        // keep track of the start of the span
        let start = self.index as usize;
        let mut end = self.index as usize;

        let mut widths = Vec::new();

        // The width of the characters
        let mut total_width: u32 = 0;

        while let Some(char_width) = self.grapheme_widths.peek() {
            if state != State::from(char_width.grapheme) {
                break;
            }

            // The char is the same 'mode', advance the iterator
            let char_width = self.grapheme_widths.next().unwrap();

            // Increment the end of the span
            end += char_width.grapheme.len();
            total_width += u32::from(char_width.display_width);
            widths.push(char_width.display_width);

            // Do not group newlinesn together, break
            if state == State::Newline {
                break;
            }
        }

        self.index = end;

        let token = Token {
            start,
            end,
            display_width: total_width,
        };

        match state {
            State::Newline => TokenKind::Newline(Some(token)),
            State::WhiteSpace => TokenKind::Optional(token),
            State::Other => TokenKind::Required(token),
        }
        .into()
    }
}

#[cfg(test)]
mod tests {

    use ttf_parser::Face;

    use crate::{grapheme_width::WithGraphemeWidth, measure::TTFParserMeasure};

    use super::*;

    #[test]
    fn one_word() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "word";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["word"]);

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn sequential_newlines() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\r\n\n\r\n\n";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["\r\n", "\n", "\r\n", "\n"]);

        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn begin_rn() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\r\na\n";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["\r\n", "a", "\n"]);

        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn newline_breaks_ws() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "  \n  ";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(words, vec!["  ", "\n", "  "]);

        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn mixed() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "at newline\n  some thing";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();
        assert_eq!(
            words,
            vec!["at", " ", "newline", "\n", "  ", "some", " ", "thing"]
        );

        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Newline(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Optional(_))));
        assert!(matches!(iter.next(), Some(TokenKind::Required(_))));
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_y() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "y̆";
        let mut iter =
            WhiteSpaceIterator::new(text.with_grapheme_width(&measure).peekable(), &measure);

        let words: Vec<&str> = iter
            .clone()
            .map(|t| t.into_token().unwrap().as_str(text))
            .collect();

        assert_eq!(words, vec!["y̆"]);

        let token = iter.next();
        assert!(matches!(token, Some(TokenKind::Required(_))));
        assert!(iter.next().is_none());
    }
}
