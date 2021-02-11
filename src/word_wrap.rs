pub trait WordWrap<'fnt, 'txt: 'fnt> {
    type Iterator: 'fnt;
    fn word_wrap(&'fnt self, text: &'txt str) -> Self::Iterator;
}

/// Provides `.wrap()` on `&str`s
///
/// The behavior of the wrapping can change depending on the `WordWrap` type passed in.
pub trait Wrap<'fnt, 'txt: 'fnt, T>
where
    T: WordWrap<'fnt, 'txt>,
{
    /// Based on the `word_wrap` provided, provides an iterator of split lines.
    fn wrap(&self, word_wrap: &'fnt T) -> T::Iterator;
}

impl<'fnt, 'txt: 'fnt, T> Wrap<'fnt, 'txt, T> for &str
where
    T: WordWrap<'fnt, 'txt>,
    T::Iterator: 'fnt,
    Self: 'txt,
{
    fn wrap(&self, word_wrap: &'fnt T) -> T::Iterator {
        word_wrap.word_wrap(self)
    }
}

pub trait WordWrapWithPosition<'fnt, 'txt: 'fnt> {
    type Iterator: 'fnt;
    fn word_wrap_with_position(&'fnt self, text: &'txt str) -> Self::Iterator;
}

/// Provides `.wrap_with_position()` on `&str`s
///
/// The behavior of the wrapping can change depending on the `WordWrap` type passed in.
pub trait WrapWithPosition<'fnt, 'txt: 'fnt, T>
where
    T: WordWrapWithPosition<'fnt, 'txt>,
{
    /// Based on the `word_wrap` provided, provides an iterator of split lines.
    fn wrap_with_position(&self, word_wrap: &'fnt T) -> T::Iterator;
}

impl<'fnt, 'txt: 'fnt, T> WrapWithPosition<'fnt, 'txt, T> for &str
where
    T: WordWrapWithPosition<'fnt, 'txt>,
    T::Iterator: 'fnt,
    Self: 'txt,
{
    fn wrap_with_position(&self, word_wrap: &'fnt T) -> T::Iterator {
        word_wrap.word_wrap_with_position(self)
    }
}
