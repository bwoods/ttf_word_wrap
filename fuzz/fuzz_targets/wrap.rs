#![no_main]
use libfuzzer_sys::fuzz_target;

use ttf_parser::Face;
use ttf_word_wrap::{TTFParserMeasure, WhiteSpaceWordWrap, Wrap};

fuzz_target!(|data: &[u8]| {
    // Load a TrueType font using `ttf_parser`
    let font_data = std::fs::read("./../test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
    let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
    let measure = TTFParserMeasure::new(&font_face);

    /*
    let mut seed = [0_u8; 8];
    data.iter()
        .copied()
        .take(8)
        .enumerate()
        .for_each(|(index, n)| seed[index] = n);

    let seed = u64::from_be_bytes(seed);
    let rand = fastrand::Rng::with_seed(seed);

    let max_width = rand.u32(..);
    */

    let max_width = 20000;
    // Set up wrapping options, split on whitespace:
    let word_wrap = WhiteSpaceWordWrap::new(max_width, &measure);

    let _ = String::from_utf8(data.to_vec()).map(|s| {
        let _ = (&s[..]).wrap(&word_wrap).collect::<Vec<&str>>();
    });
});
