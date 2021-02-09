pub mod whitespace;

#[derive(PartialEq, Clone, Debug)]
pub enum TokenKind {
    /// The token must be used.
    Required,

    /// The token may be removed at a line break.
    Optional,

    /// The token causes a newline
    Newline,
}

#[derive(Clone, Debug)]
pub struct Token {
    /// The `TokenKind` of the token.
    pub kind: TokenKind,

    /// Where the token starts in `text`
    pub start: usize,

    /// Where the token ends in `text`
    pub end: usize,

    /// The width of the token for the given font
    pub width: u32,
}

impl Token {
    pub fn split_at(self, index: usize) -> (Token, Token) {
        unimplemented!()
    }
}

/*
impl<'a> std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.text[self.start..self.end])
    }
}
*/
