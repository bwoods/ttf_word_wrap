#![no_main]
use libfuzzer_sys::fuzz_target;

use ttf_parser::Face;
use ttf_word_wrap::{TTFParserMeasure, WhiteSpaceWordWrap, Wrap};

fuzz_target!(|s: String| {
    // Load a TrueType font using `ttf_parser`
    let font_data = std::fs::read("./../test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
    let font_face = Face::parse(&font_data, 0).expect("TTF should be valid");
    let measure = TTFParserMeasure::new(&font_face);

    let max_width = 20000;
    // Set up wrapping options, split on whitespace:
    let word_wrap = WhiteSpaceWordWrap::new(max_width, &measure);

    let _ = (&s[..]).wrap(&word_wrap).collect::<Vec<&str>>();
});
