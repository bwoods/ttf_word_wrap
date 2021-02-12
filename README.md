# Word wrap for variable width fonts.

Can provide split lines or positions for each character.

#### Lines Split

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

#### Positions for each character

```rust
use ttf_parser::Face;
use ttf_word_wrap::{WrapWithPosition, WhiteSpaceWordWrap, TTFParserMeasure, Position};

// Load a TrueType font using `ttf_parser`
let font_data = std::fs::read("./test_fonts/Roboto-Regular.ttf").expect("TTF should exist");
let font_face = Face::from_slice(&font_data, 0).expect("TTF should be valid");
let measure = TTFParserMeasure::new(&font_face);

// Set up wrapping options, split on whitespace:
let word_wrap = WhiteSpaceWordWrap::new(20000, &measure);

// Use the `Wrap` trait and split the `&str`
let poem = "Mary had a little lamb whose fleece was white as snow";
let positions: Vec<Position> = poem.wrap_with_position(&word_wrap).collect();

// offset is in the unit (em) of the TTFParserMeasure.
assert_eq!(positions[0], Position { ch: 'M', line: 0, offset: 0 });
```
