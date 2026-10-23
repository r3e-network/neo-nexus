//! The console stylesheet.
//!
//! It lives in `console.css` beside this module rather than in a `static/`
//! directory: NeoNexus has no asset pipeline, and `include_str!` bakes the file
//! into the binary exactly as a string literal would. The security headers hash
//! this same constant for `style-src`, so the browser accepts the stylesheet
//! that ships with the binary and nothing else.
//!
//! Keeping the rules in a `.css` file means the editor and the diff both see
//! CSS, and this module stops being the one file that owns the whole interface.

pub const CSS: &str = include_str!("console.css");
