# Word wrap for variable width fonts.

An example:

```rust
use ttf_parser::Face;
use ttf_word_wrap::{Wrap, WhiteSpaceWordWrap};

// Load a TrueType font using `ttf_parser`
let font_data = std::fs::read("./test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");

// Set up wrapping options, split on whitespace:
let word_wrap = WhiteSpaceWordWrap::new(20000, &font_face);

// Use the `Wrap` trait and split the `&str`
let poem = "Mary had a little lamb whose fleece was white as snow";
let lines: Vec<&str> = poem.wrap(&word_wrap).collect();
assert_eq!(lines[0], "Mary had a little lamb");
```
