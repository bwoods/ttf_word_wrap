use std::ops::Range;

pub mod whitespace;

//pub type Tokenizer<'a> = dyn Fn(&'a str) -> Box<dyn Iterator<Item = Token> + 'a>;

pub trait Tokenizer: std::fmt::Debug {
    fn tokenize<'a>(&self, text: &'a str) -> Box<dyn Iterator<Item = Token> + 'a>;
}

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
    pub kind: TokenKind,
    pub span: Range<usize>,
}
