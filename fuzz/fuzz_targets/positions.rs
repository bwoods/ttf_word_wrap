#![no_main]
use libfuzzer_sys::fuzz_target;

use ttf_parser::Face;
use ttf_word_wrap::{Position, TTFParserMeasure, WhiteSpaceWordWrap, WrapWithPosition};

fuzz_target!(|data: &[u8]| {
    // Load a TrueType font using `ttf_parser`
    let font_data = std::fs::read("./../test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
    let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
    let measure = TTFParserMeasure::new(&font_face);

    // Set up wrapping options, split on whitespace:
    let word_wrap = WhiteSpaceWordWrap::new(20000, &measure);

    let _ = String::from_utf8(data.to_vec()).map(|s| {
        let _ = (&s[..])
            .wrap_with_position(&word_wrap)
            .collect::<Vec<Position>>();
    });
});
