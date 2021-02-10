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
