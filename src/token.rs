use crate::display_width::DisplayWidth;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Kind {
    Required,
    Optional,
    Newline,
}

impl Kind {
    pub fn token<'a>(&self, token: Token<'a>) -> TokenKind<'a> {
        match self {
            Kind::Required => TokenKind::Required(token),
            Kind::Optional => TokenKind::Optional(token),
            Kind::Newline => TokenKind::Newline(token),
        }
    }
}

/// The kind of the token.
///
/// `Optional` tokens may be omitted from lines if they occur at the beginning or end of a line.
/// `Newline` tokens cause a newline.
#[derive(Clone, Debug)]
pub enum TokenKind<'a> {
    /// The token must be used.
    Required(Token<'a>),

    /// The token may be removed at a line break.
    Optional(Token<'a>),

    /// The token causes a newline
    Newline(Token<'a>),
}

impl<'a> TokenKind<'a> {
    pub fn is_required(&self) -> bool {
        matches!(self, TokenKind::Required(_))
    }

    pub fn is_optional(&self) -> bool {
        matches!(self, TokenKind::Optional(_))
    }

    pub fn is_newline(&self) -> bool {
        matches!(self, TokenKind::Newline(_))
    }

    pub fn width(&self) -> u32 {
        match self {
            TokenKind::Required(token) => token.width,
            TokenKind::Optional(token) => token.width,
            TokenKind::Newline(token) => token.width,
        }
    }

    pub fn into_token(self) -> Token<'a> {
        match self {
            TokenKind::Required(token) | TokenKind::Optional(token) | TokenKind::Newline(token) => {
                token
            }
        }
    }

    pub fn kind(&self) -> Kind {
        match self {
            TokenKind::Required(_) => Kind::Required,
            TokenKind::Optional(_) => Kind::Optional,
            TokenKind::Newline(_) => Kind::Newline,
        }
    }
}

/// A Token is a portion of a &str
#[derive(Clone, Debug)]
pub struct Token<'a> {
    pub display_width: &'a dyn DisplayWidth,

    /// Token points to data in this &str
    pub text: &'a str,

    /// Where the token starts in `text`
    pub start: usize,

    /// Where the token ends in `text`
    pub end: usize,

    /// The width of the token for the given font
    pub width: u32,
}

impl<'a> Token<'a> {
    /// Creates a new token for the whole `&str` with the given `kind` and `font_face`.
    pub fn new(text: &'a str, display_width: &'a dyn DisplayWidth) -> Self {
        let width = display_width.measure_str(text);

        Self {
            display_width,
            text,
            start: 0,
            end: text.len(),
            width,
        }
    }

    /// Subdivides the token at `display_width`.
    pub fn split_at(&self, display_width: u32) -> (Option<Token<'a>>, Option<Token<'a>>) {
        // Should never happen...
        if self.start == self.end {
            return (None, None);
        }

        // Optimize the case where the token fits in the display_width
        if self.width < display_width {
            return (Some(self.clone()), None);
        }

        let mut head_width: u32 = 0;
        let mut index = 0;

        let mut chars = self.text.chars();

        while let Some(ch) = chars.next() {
            let ch_display_width = self.display_width.measure_char(ch);
            let next_width = head_width + u32::from(ch_display_width);

            if next_width > display_width {
                break;
            }

            head_width = next_width;
            index += ch.len_utf8();
        }

        // Could not find any chars that fit in the display_width
        if head_width == 0 {
            return (None, Some(self.clone()));
        }

        let mut head = self.clone();
        let mut tail = self.clone();

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
    use ttf_parser::Face;

    use crate::display_width::TTFParserDisplayWidth;

    use super::*;

    #[test]
    fn full_width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let token = Token::new("1234567890", &display_width);

        let (head, tail) = token.split_at(1_000_000);
        assert_eq!("1234567890", head.unwrap().to_string());
        assert!(tail.is_none());
    }

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let token = Token::new("1234567890", &display_width);

        // the number 1 is much larger than the display width of 100
        let (head, tail) = token.split_at(100);
        assert!(head.is_none());
        assert_eq!("1234567890", tail.unwrap().to_string());
    }

    #[test]
    fn split_even() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let display_width = TTFParserDisplayWidth::new(&font_face);

        let token = Token::new("1234567890", &display_width);

        let (head, tail) = token.split_at(6000);
        assert_eq!("12345", head.unwrap().to_string());
        assert_eq!("67890", tail.unwrap().to_string());
    }
}
