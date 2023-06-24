use unicode_segmentation::UnicodeSegmentation;

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

    /// Creates a new Token
    pub fn measure(text: &str, measure: &dyn Measure) -> Token {
        let display_width = measure.str(&text);
        Self {
            start: 0,
            end: text.len(),
            display_width,
        }
    }

    /// Subdivides the token after a number of graphemes.
    pub fn split_at_grapheme(
        &self,
        graphemes: usize,
        text: &str,
        measure: &dyn Measure,
    ) -> (Option<Token>, Option<Token>) {
        // Our little slice of the world
        let slice = &text[self.start..self.end];

        let index = slice.graphemes(true).take(graphemes).map(|s| s.len()).sum();

        self.split_at(index, text, measure)
    }

    pub fn split_at(
        &self,
        index: usize,
        text: &str,
        measure: &dyn Measure,
    ) -> (Option<Token>, Option<Token>) {
        let index = self.start + index;

        let mut head = self.clone();
        head.end = index;
        head.display_width = measure.str(&text[head.start..head.end]);

        let mut tail = self.clone();
        tail.start = index;
        tail.display_width = measure.str(&text[tail.start..tail.end]);

        match (head.start == head.end, tail.start == tail.end) {
            (true, true) => (None, None),
            (false, true) => (Some(head), None),
            (true, false) => (None, Some(tail)),
            (false, false) => (Some(head), Some(tail)),
        }
    }

    /// Subdivides the token at `display_width`.
    pub fn split_at_width(
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

        let mut chars = text[self.start..self.end].chars();

        while let Some(ch) = chars.next() {
            // char widths that are not known will be placed on the same line
            let ch_display_width = measure.char(ch).unwrap_or_default();
            let next_width = head_width + u32::from(ch_display_width);

            if next_width > display_width {
                break;
            }

            head_width = next_width;
            index += ch.len_utf8();
        }

        self.split_at(index, text, measure)
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
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        let (head, tail) = token.split_at_width(1_000_000, text, &measure);
        assert_eq!("1234567890", head.unwrap().as_str(text));
        assert!(tail.is_none());
    }

    #[test]
    fn too_narrow() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        // the number 1 is much larger than the display width of 100
        let (head, tail) = token.split_at_width(100, text, &measure);
        assert!(head.is_none());
        assert_eq!("1234567890", tail.unwrap().as_str(&text));
    }

    #[test]
    fn split_even() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "1234567890";
        let token = Token::measure(text, &measure);

        let (head, tail) = token.split_at_width(6000, text, &measure);
        assert_eq!("12345", head.unwrap().as_str(text));
        assert_eq!("67890", tail.unwrap().as_str(text));
    }

    #[test]
    fn fuzz_wrap_1() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "\r\n[wwwwwwwwwwwww";
        let token = Token {
            start: 2,
            end: 16,
            display_width: 20550,
        };

        let (head, tail) = token.split_at_width(20000, text, &measure);
        assert_eq!("[wwwwwwwwwwww", head.unwrap().as_str(text));
        assert_eq!("w", tail.unwrap().as_str(text));
    }

    #[test]
    fn hello() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "今日は!";

        let token = Token::measure(text, &measure);

        let (head, tail) = token.split_at_grapheme(1, text, &measure);
        assert_eq!("今", head.unwrap().as_str(text));
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!("日", head.unwrap().as_str(text));
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!("は", head.unwrap().as_str(text));
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!("!", head.unwrap().as_str(text));

        assert!(tail.is_none());
    }

    #[test]
    fn width() {
        let font_data = crate::tests::read_font();
        let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let text = "今日は!";

        let token = Token::measure(text, &measure);
        assert_eq!(527, token.display_width);

        // the Roboto font does not have these Japanese glyphs.
        let (head, tail) = token.split_at_grapheme(1, text, &measure);
        assert_eq!(0, head.unwrap().display_width);
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!(0, head.unwrap().display_width);
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!(0, head.unwrap().display_width);
        let (head, tail) = tail.unwrap().split_at_grapheme(1, text, &measure);
        assert_eq!(527, head.unwrap().display_width);

        assert!(tail.is_none());
    }
}
