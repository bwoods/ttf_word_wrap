use crate::{
    line_builder::{DefaultLineBuilder, LineBuilder},
    tokenize::{whitespace::WhiteSpaceTokenizer, Tokenizer},
};

/// Options for `WordWrap`
#[derive(Debug)]
pub struct Options {
    max_width: u32,
    tokenizer: Box<dyn Tokenizer>,
    line_builder: Box<dyn LineBuilder>,
}

impl Options {
    /// Creates a new Options
    ///
    /// By default Options will have a whitespace tokenizer.
    ///
    /// `max_width` is defined in EMs
    pub fn builder(max_width: u32) -> OptionsBuilder {
        OptionsBuilder {
            max_width,
            tokenizer: None,
            line_builder: None,
        }
    }

    /// The `Tokenizer`
    pub fn tokenizer(&mut self) -> &mut dyn Tokenizer {
        self.tokenizer.as_mut()
    }

    /// The `LineBuilder`
    pub fn line_builder(&mut self) -> &mut dyn LineBuilder {
        self.line_builder.as_mut()
    }
}

#[derive(Debug)]
pub struct OptionsBuilder {
    max_width: u32,
    tokenizer: Option<Box<dyn Tokenizer>>,
    line_builder: Option<Box<dyn LineBuilder>>,
}

impl OptionsBuilder {
    /// Sets the tokenizer
    pub fn with_tokenizer<T>(mut self, tokenizer: T) -> Self
    where
        T: Tokenizer + 'static,
    {
        self.tokenizer = Some(Box::new(tokenizer));
        self
    }

    /// Sets the line builder
    pub fn with_line_builder<T>(mut self, line_builder: T) -> Self
    where
        T: LineBuilder + 'static,
    {
        self.line_builder = Some(Box::new(line_builder));
        self
    }

    pub fn build(self) -> Options {
        let Self {
            line_builder,
            tokenizer,
            max_width,
        } = self;

        let tokenizer = tokenizer.unwrap_or_else(|| Box::new(WhiteSpaceTokenizer::new()));
        let line_builder = line_builder.unwrap_or_else(|| Box::new(DefaultLineBuilder::new()));

        Options {
            max_width,
            tokenizer,
            line_builder,
        }
    }
}
