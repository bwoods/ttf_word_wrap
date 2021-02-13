use std::collections::HashMap;

use ttf_parser::Face;

/// Implementing this allows overriding of how glyphs are measured.
pub trait Measure: std::fmt::Debug {
    /// Measures the display width of `text`.
    fn str(&self, text: &str) -> u32;

    /// Measures the display width of `c`.
    ///
    /// Returns `None` if the width is not known.
    fn char(&self, c: char) -> Option<u16>;
}

/// Implements measuring glyphs via `ttf_parser`
#[derive(Clone, Debug)]
pub struct TTFParserMeasure<'a> {
    face: &'a Face<'a>,
    cache: HashMap<char, u16>,
}

impl<'a> TTFParserMeasure<'a> {
    /// Creates a new TTFParserMeasure for the font `face`.
    pub fn new(face: &'a Face<'a>) -> Self {
        Self {
            face,
            cache: HashMap::new(),
        }
    }
}

impl<'a> Measure for TTFParserMeasure<'a> {
    fn str(&self, text: &str) -> u32 {
        text.chars()
            .map(|c| u32::from(self.char(c).unwrap_or_default()))
            .sum()
    }

    #[inline]
    fn char(&self, c: char) -> Option<u16> {
        self.face
            .glyph_index(c)
            .map(|glyph_id| self.face.glyph_hor_advance(glyph_id))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let dw = TTFParserMeasure::new(&font_face);

        let text = "aoeu";
        let width = dw.str(text);
        assert_eq!(width, 4496);
    }

    #[test]
    fn test_char() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let dw = TTFParserMeasure::new(&font_face);

        let text = "a";
        let width = dw.char(text.chars().next().unwrap()).unwrap();
        assert_eq!(width, 1114);
    }

    #[test]
    fn caverns() {
        let font_data = crate::tests::read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let dw = TTFParserMeasure::new(&font_face);

        let text = "caverns are not for the";
        let width = dw.str(text);
        assert_eq!(width, 20483);
    }
}
