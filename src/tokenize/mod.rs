use crate::tokenize::span::Span;

pub mod span;
pub mod whitespace;

pub trait Tokenizer: std::fmt::Debug {
    fn tokenize<'a>(&mut self, text: &'a str) -> Box<dyn Iterator<Item = Token> + 'a>;
}

#[derive(Clone, Debug)]
pub enum TokenKind {
    /// The token must be used.
    Required,

    /// The token may be removed at a line break.
    Optional,
}

#[derive(Clone, Debug)]
pub struct Token {
    kind: TokenKind,
    span: Span,
}
