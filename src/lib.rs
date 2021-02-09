//! A library for wrapping text based on a font and a given maximum line width.
//!
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

mod char_width;
mod partial_tokens;
mod tokenize;

// mod line_builder;
// mod word_wrap;

// pub use line_builder::DefaultLineIterator;
// pub use word_wrap::WordWrap;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ttf_parser::Face;

    use super::*;

    pub fn read_font() -> Vec<u8> {
        let font_path: PathBuf = [
            env!("CARGO_MANIFEST_DIR"),
            "test_fonts",
            "Roboto-Regular.ttf",
        ]
        .iter()
        .collect();
        std::fs::read(font_path).expect("TTF should exist")
    }

    //    #[test]
    //    fn one_line() {
    //        let font_data = read_font();
    //        let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
    //
    //        let actual: Vec<&str> =
    //            WordWrap::wrap("this is a test \n of the word wrap", 3000, font_face).collect();
    //
    //        let expected = vec!["this is a test", "of the word wrap"];
    //
    //        assert_eq!(expected, actual);
    //    }
}
