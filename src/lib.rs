//! A library for wrapping text based on a font and a given maximum line width.
//!
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]
#![doc(test(attr(deny(rust_2018_idioms, warnings))))]
#![doc(test(attr(allow(unused_extern_crates, unused_variables))))]

pub mod char_width;
pub mod line;
pub mod partial_tokens;
pub mod token;
pub mod whitespace;
pub mod word_wrap;

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use ttf_parser::Face;

    use crate::{whitespace::WhiteSpaceWordWrap, word_wrap::Wrap};

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

        let wsww = WhiteSpaceWordWrap::new(20000, &font_face);

        let actual: Vec<&str> =
            "The nethermost caverns are not for the fathoming of eyes that see; for their marvels are strange and terrific. Cursed the ground where dead thoughts live new and oddly bodied, and evil the mind that is held by no head. Wisely did Ibn Schacabao say, that happy is the tomb where no wizard hath lain, and happy the town at night whose wizards are all ashes. For it is of old rumour that the soul of the devil-bought hastes not from his charnel clay, but fats and instructs the very worm that gnaws; till out of corruption horrid life springs, and the dull scavengers of earth wax crafty to vex it and swell monstrous to plague it. Great holes are digged where earth's pores ought to suffice, and things have learnt to walk that ought to crawl.".wrap(&wsww).collect();

        let expected = vec!["this is a test", "of the word wrap"];

        assert_eq!(expected, actual);
    }
}
