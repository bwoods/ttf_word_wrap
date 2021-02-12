use crate::measure::Measure;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Kind {
    Required,
    Optional,
    Newline,
}

impl Kind {
    pub fn token(&self, token: Token) -> TokenKind {
        match self {
            Kind::Required => TokenKind::Required(token),
            Kind::Optional => TokenKind::Optional(token),
            Kind::Newline => TokenKind::Newline(Some(token)),
        }
    }
}

/// The kind of the token.
///
/// `Optional` tokens may be omitted from lines if they occur at the beginning or end of a line.
/// `Newline` tokens cause a newline.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
    /// The token must be used.
    Required(Token),

    /// The token may be removed at a line break.
    Optional(Token),

    /// The token causes a newline
    Newline(Option<Token>),
}

impl TokenKind {
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
            TokenKind::Required(token) => token.display_width,
            TokenKind::Optional(token) => token.display_width,
            TokenKind::Newline(Some(token)) => token.display_width,
            TokenKind::Newline(None) => 0,
        }
    }

    pub fn into_token(self) -> Option<Token> {
        match self {
            TokenKind::Required(token) | TokenKind::Optional(token) => Some(token),
            TokenKind::Newline(token) => token,
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Token {
    /// Where the token starts in `text`
    pub start: usize,

    /// Where the token ends in `text`
    pub end: usize,

    /// The width of the token for the given font
    pub display_width: u32,
}

impl Token {
    /// Creates a new token for the whole `&str` with the given `kind` and `font_face`.
    pub fn new(start: usize, end: usize, display_width: u32) -> Self {
        Self {
            start,
            end,
            display_width,
        }
    }

    pub fn measure<T>(text: &str, measure: &T) -> Token
    where
        T: Measure,
    {
        let display_width = measure.str(&text);
        Self {
            start: 0,
            end: text.len(),
            display_width,
        }
    }

    /// Subdivides the token at `display_width`.
    pub fn split_at(
        &self,
        display_width: u32,
        text: &str,
        measure: &dyn Measure,
    ) -> (Option<Token>, Option<Token>) {
        // Should never happen...
        if self.start == self.end {
            return (None, None);
        }

        // Optimize the case where the token fits in the display_width
        if self.display_width < display_width {
            return (Some(self.clone()), None);
        }

        let mut head_width: u32 = 0;
        let mut index = 0;

        let mut chars = text.chars();

        while let Some(ch) = chars.next() {
            let ch_display_width = measure.char(ch);
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
        head.display_width = head_width;

        tail.start += index;
        tail.display_width -= head_width;

        (Some(head), Some(tail))
    }

    pub fn as_str(self, text: &str) -> &str {
        &text[self.start..self.end]
    }
}

#[cfg(test)]
mod tests {
    use ttf_parser::Face;

    use crate::TTFParserMeasure;

    use super::*;

    #[test]
    fn full_width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        let (head, tail) = token.split_at(1_000_000, text, &measure);
        assert_eq!("1234567890", head.unwrap().as_str(text));
        assert!(tail.is_none());
    }

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        // the number 1 is much larger than the display width of 100
        let (head, tail) = token.split_at(100, text, &measure);
        assert!(head.is_none());
        assert_eq!("1234567890", tail.unwrap().as_str(&text));
    }

    #[test]
    fn split_even() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        let (head, tail) = token.split_at(6000, text, &measure);
        assert_eq!("12345", head.unwrap().as_str(text));
        assert_eq!("67890", tail.unwrap().as_str(text));
    }
}
