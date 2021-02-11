use std::collections::HashMap;

use ttf_parser::Face;

/// Implementing this allows overriding of how glyphs are measured.
pub trait DisplayWidth: std::fmt::Debug {
    /// Measures the display width of `text`.
    fn measure_str(&self, text: &str) -> u32;

    /// Measures the display width of `c`.
    fn measure_char(&self, c: char) -> u16;
}

/// Implements measuring glyphs via `ttf_parser`
#[derive(Clone, Debug)]
pub struct TTFParserDisplayWidth<'a> {
    face: &'a Face<'a>,
    cache: HashMap<char, u16>,
}

impl<'a> TTFParserDisplayWidth<'a> {
    /// Creates a new TTFParserDisplayWidth for the font `face`.
    pub fn new(face: &'a Face<'a>) -> Self {
        Self {
            face,
            cache: HashMap::new(),
        }
    }
}

impl<'a> DisplayWidth for TTFParserDisplayWidth<'a> {
    fn measure_str(&self, text: &str) -> u32 {
        text.chars().map(|c| u32::from(self.measure_char(c))).sum()
    }

    #[inline]
    fn measure_char(&self, c: char) -> u16 {
        self.face
            .glyph_index(c)
            .map(|glyph_id| self.face.glyph_hor_advance(glyph_id))
            .flatten()
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let dw = TTFParserDisplayWidth::new(&font_face);

        let text = "aoeu";
        let width = dw.measure_str(text);
        assert_eq!(width, 4496);
    }

    #[test]
    fn test_char() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let dw = TTFParserDisplayWidth::new(&font_face);

        let text = "a";
        let width = dw.measure_char(text.chars().next().unwrap());
        assert_eq!(width, 1114);
    }
}
