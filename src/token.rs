use ttf_parser::Face;

use crate::char_width::WithCharWidth;

/// The kind of the token.
///
/// `Optional` tokens may be omitted from lines if they occur at the beginning or end of a line.
/// `Newline` tokens cause a newline.
#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    /// The token must be used.
    Required,

    /// The token may be removed at a line break.
    Optional,

    /// The token causes a newline
    Newline,
}

/// A Token is a portion of a &str
#[derive(Clone, Debug)]
pub struct Token<'a> {
    /// Token points to data in this &str
    pub text: &'a str,

    /// The font of the token
    pub font_face: &'a Face<'a>,

    /// The `TokenKind` of the token.
    pub kind: TokenKind,

    /// Where the token starts in `text`
    pub start: usize,

    /// Where the token ends in `text`
    pub end: usize,

    /// The width of the token for the given font
    pub width: u32,
}

impl<'a> Token<'a> {
    /// Creates a new token for the whole `&str` with the given `kind` and `font_face`.
    pub fn new(text: &'a str, kind: TokenKind, font_face: &'a Face<'a>) -> Self {
        let width = text
            .with_char_width(font_face)
            .map(|char_width| u32::from(char_width.display_width))
            .sum();

        Self {
            text,
            font_face,
            start: 0,
            end: text.len(),
            width,
            kind,
        }
    }

    /// Subdivides the token at `display_width`.
    pub fn split_at(self, display_width: u32) -> (Option<Token<'a>>, Option<Token<'a>>) {
        // Should never happen...
        if self.start == self.end {
            return (None, None);
        }

        // Optimize the case where the token fits in the display_width
        if self.width < display_width {
            return (Some(self), None);
        }

        let mut head_width: u32 = 0;
        let mut index = 0;

        let mut char_widths = self.text.with_char_width(self.font_face);
        while let Some(char_width) = char_widths.next() {
            let next_width = head_width + u32::from(char_width.display_width);

            if next_width > display_width {
                break;
            }

            head_width = next_width;
            index += char_width.ch.len_utf8();
        }

        // Could not find any chars that fit in the display_width
        if head_width == 0 {
            return (None, Some(self));
        }

        let mut head = self.clone();
        let mut tail = self;

        head.end = head.start + index;
        head.width = head_width;

        tail.start += index;
        tail.width -= head_width;

        (Some(head), Some(tail))
    }
}

impl<'a> std::fmt::Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text[self.start..self.end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let token = Token::new("1234567890", TokenKind::Required, &font_face);

        let (head, tail) = token.split_at(1_000_000);
        assert_eq!("1234567890", head.unwrap().to_string());
        assert!(tail.is_none());
    }

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let token = Token::new("1234567890", TokenKind::Required, &font_face);

        // the number 1 is much larger than the display width of 100
        let (head, tail) = token.split_at(100);
        assert!(head.is_none());
        assert_eq!("1234567890", tail.unwrap().to_string());
    }

    #[test]
    fn split_even() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

        let token = Token::new("1234567890", TokenKind::Required, &font_face);

        let (head, tail) = token.split_at(6000);
        assert_eq!("12345", head.unwrap().to_string());
        assert_eq!("67890", tail.unwrap().to_string());
    }
}
