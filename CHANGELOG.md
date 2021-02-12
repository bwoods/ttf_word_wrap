# 0.2.0

#### Added:

`wrap_with_position` on &str that returns a `Position` for each `char`.
A trait `Measure` and an implementation based on `ttf_parser` called `TTFParserMeasure`.

#### Migration:

A type implementing `Measure` must be passed into `wrap()` instead of `&Face`.

# 0.1.0

Initial Release, provides word wrapping of &str at a given display width.