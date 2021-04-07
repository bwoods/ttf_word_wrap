# 0.5.0

Added the glyph width to Position.

# 0.4.2

Updating ttf_parser to 0.12.0

# 0.4.1

Changed internal recursion to iteration.
Improved fuzzing.
Graphemes that are wider than the max_width will be returned on their own lines.

# 0.4.0

Changed the return type of `wrap_with_position()` from `Position` to `CharPosition`.

# 0.3.0

Fixed a bug when tokenizing "y̆".
Using `unicode_segmentation` to segment graphemes.

# 0.2.2

#### Fixed

Fuzzer found a bug in Token::split_at(), it has been fixed.

# 0.2.1

#### Fixed

Examples in README.md now work.

# 0.2.0

#### Added:

`wrap_with_position` on &str that returns a `Position` for each `char`.
A trait `Measure` and an implementation based on `ttf_parser` called `TTFParserMeasure`.

#### Migration:

A type implementing `Measure` must be passed into `wrap()` instead of `&Face`.

# 0.1.0

Initial Release, provides word wrapping of &str at a given display width.
