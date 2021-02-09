pub trait WordWrap<'fnt, 'txt: 'fnt> {
    type Iterator: 'fnt;
    fn word_wrap(&'fnt self, text: &'txt str) -> Self::Iterator;
}

pub trait Wrap<'fnt, 'txt: 'fnt, T>
where
    T: WordWrap<'fnt, 'txt>,
{
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
