//! Useful when creating new wrapping iterators.

use crate::measure::Measure;

use unicode_segmentation::{Graphemes, UnicodeSegmentation};

/// A char and it's display width, along wtith the `font_face` and `&str` it came from.
#[derive(Clone, Debug)]
pub struct GraphemeWidth<'a> {
    pub grapheme: &'a str,
    pub display_width: u32,
}

pub trait WithGraphemeWidth {
    fn with_grapheme_width<'a>(&'a self, measure: &'a dyn Measure) -> GraphemeWidthIterator<'a>;
}

impl WithGraphemeWidth for str {
    fn with_grapheme_width<'a>(&'a self, measure: &'a dyn Measure) -> GraphemeWidthIterator<'a> {
        let graphemes = self.graphemes(true);
        GraphemeWidthIterator { measure, graphemes }
    }
}

#[derive(Clone)]
pub struct GraphemeWidthIterator<'a> {
    measure: &'a dyn Measure,
    graphemes: Graphemes<'a>,
}

impl<'a> std::fmt::Debug for GraphemeWidthIterator<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphemeWidthIterator").finish()
    }
}

impl<'a> Iterator for GraphemeWidthIterator<'a> {
    type Item = GraphemeWidth<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let grapheme = self.graphemes.next()?;

        let display_width = self.measure.str(grapheme);

        Some(GraphemeWidth {
            grapheme,
            display_width,
        })
    }
}
