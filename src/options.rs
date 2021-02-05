use crate::tokenize::{whitespace::WhiteSpaceTokenizer, Tokenizer};

/// Options for `WordWrap`
#[derive(Debug)]
pub struct Options {
    max_width: u32,
    tokenizer: Box<dyn Tokenizer>,
}

impl Options {
    /// Creates a new Options
    ///
    /// By default Options will have a whitespace tokenizer.
    ///
    /// `max_width` is defined in EMs
    pub fn new(max_width: u32) -> Self {
        let tokenizer = Box::new(WhiteSpaceTokenizer::new());

        Self {
            max_width,
            tokenizer,
        }
    }

    /// The current `Tokenizer`
    pub fn tokenizer(&mut self) -> &mut dyn Tokenizer {
        self.tokenizer.as_mut()
    }

    /// Override the default tokenizer
    pub fn with_tokenizer<T>(mut self, tokenizer: T) -> Self
    where
        T: Tokenizer + 'static,
    {
        self.tokenizer = Box::new(tokenizer);
        self
    }
}
