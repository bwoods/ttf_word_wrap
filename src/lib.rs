//! A library for wrapping text based on a font and a given maximum line width.
//!
//!
//! A simple example using [`ttf_parser`](https://crates.io/crates/ttf-parser) as the font parser.
//!
//!```
//! use ttf_parser::Face;
//! use ttf_word_wrap::{Wrap, WhiteSpaceWordWrap, TTFParserMeasure};
//!
//! // Load a TrueType font using `ttf_parser`
//! let font_data = std::fs::read("./test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
//! let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
//! let measure = TTFParserMeasure::new(&font_face);
//!
//! // Set up wrapping options, split on whitespace:
//! let word_wrap = WhiteSpaceWordWrap::new(20000, &measure);
//!
//! // Use the `Wrap` trait and split the `&str`
//! let poem = "Mary had a little lamb whose fleece was white as snow";
//! let lines: Vec<&str> = poem.wrap(&word_wrap).collect();
//! assert_eq!(lines[0], "Mary had a little lamb");
//!```
//!
//! A more complicated example that returns `Position`s of the glyphs.
//!
//!```
//! use ttf_parser::Face;
//! use ttf_word_wrap::{WrapWithPosition, WhiteSpaceWordWrap, TTFParserMeasure, Position};
//!
//! // Load a TrueType font using `ttf_parser`
//! let font_data = std::fs::read("./test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
//! let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
//! let measure = TTFParserMeasure::new(&font_face);
//!
//! // Set up wrapping options, split on whitespace:
//! let word_wrap = WhiteSpaceWordWrap::new(20000, &measure);
//!
//! // Use the `Wrap` trait and split the `&str`
//! let poem = "Mary had a little lamb whose fleece was white as snow";
//! let positions: Vec<Position> = poem.wrap_with_position(&word_wrap).collect();
//!
//! // offset is in the unit (em) of the TTFParserMeasure.
//! assert_eq!(positions[0], Position { ch: 'M', line: 0, offset: 0 });
//!```
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

mod char_width;
mod line;
mod line_break;
mod measure;
mod partial_tokens;
mod position;
mod token;
mod whitespace;
mod whitespace_wordwrap;
mod wordwrap;

pub use measure::{Measure, TTFParserMeasure};
pub use position::Position;
pub use whitespace_wordwrap::WhiteSpaceWordWrap;
pub use wordwrap::{Wrap, WrapWithPosition};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ttf_parser::Face;

    use crate::{
        measure::TTFParserMeasure, whitespace_wordwrap::WhiteSpaceWordWrap, wordwrap::Wrap,
        Position, WrapWithPosition,
    };

    pub fn read_font<'a>() -> Vec<u8> {
        let font_path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "test_fonts",
            "Roboto-Regular.ttf",
        ]
        .iter()
        .collect();
        std::fs::read(font_path).expect("TTF should exist")
    }

    #[test]
    fn nomicon() {
        let font_data = read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let wsww = WhiteSpaceWordWrap::new(20000, &measure);

        let actual: Vec<&str> =
            "The nethermost caverns are not for the fathoming of eyes that see; for their marvels are strange and terrific. Cursed the ground where dead thoughts live new and oddly bodied, and evil the mind that is held by no head. Wisely did Ibn Schacabao say, that happy is the tomb where no wizard hath lain, and happy the town at night whose wizards are all ashes. For it is of old rumour that the soul of the devil-bought hastes not from his charnel clay, but fats and instructs the very worm that gnaws; till out of corruption horrid life springs, and the dull scavengers of earth wax crafty to vex it and swell monstrous to plague it. Great holes are digged where earth's pores ought to suffice, and things have learnt to walk that ought to crawl.".wrap(&wsww).collect();

        let expected = vec![
            "The nethermost",
            "caverns are not for",
            "the fathoming of eyes",
            "that see; for their",
            "marvels are strange",
            "and terrific. Cursed",
            "the ground where",
            "dead thoughts live",
            "new and oddly",
            "bodied, and evil the",
            "mind that is held by",
            "no head. Wisely did",
            "Ibn Schacabao say,",
            "that happy is the",
            "tomb where no",
            "wizard hath lain, and",
            "happy the town at",
            "night whose wizards",
            "are all ashes. For it is",
            "of old rumour that the",
            "soul of the",
            "devil-bought hastes",
            "not from his charnel",
            "clay, but fats and",
            "instructs the very",
            "worm that gnaws; till",
            "out of corruption",
            "horrid life springs,",
            "and the dull",
            "scavengers of earth",
            "wax crafty to vex it",
            "and swell monstrous",
            "to plague it. Great",
            "holes are digged",
            "where earth\'s pores",
            "ought to suffice, and",
            "things have learnt to",
            "walk that ought to",
            "crawl.",
        ];

        assert_eq!(expected, actual);
    }

    #[test]
    fn nomicon_positions() {
        let font_data = read_font();
        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
        let measure = TTFParserMeasure::new(&font_face);

        let wsww = WhiteSpaceWordWrap::new(20000, &measure);

        let mut positions = "The nethermost caverns are not for the fathoming of eyes that see;"
            .wrap_with_position(&wsww);

        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'T',
                line: 0,
                offset: 0
            },
            token
        );

        // advance some
        (0..30).for_each(|_| {
            positions.next().unwrap();
        });
        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: 'o',
                line: 1,
                offset: 15233
            },
            token
        );

        // advance some more
        (0..30).for_each(|_| {
            positions.next().unwrap();
        });
        let token = positions.next().unwrap();
        assert_eq!(
            Position {
                ch: ';',
                line: 3,
                offset: 7313
            },
            token
        );
    }
}
