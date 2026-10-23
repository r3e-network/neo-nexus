//! Browser assets embedded in the Rust binary.
//!
//! NeoNexus deliberately has no frontend build toolchain or static asset
//! directory, and `--source-purity` enforces that: a `.css` file here would be
//! reported as a frontend source outside the pure Rust boundary. The stylesheet
//! is therefore Rust string literals split across `style/`, joined once at
//! runtime; the progressive enhancement script is a Rust module for the same
//! reason — the security headers hash each of them as one string.
//!
//! Neither is a deployment artifact to copy alongside the binary, and none of
//! the files is large enough to be hard to navigate.

mod script;
mod style;
mod theme;

pub use script::SCRIPT;
pub use style::css;
pub use theme::{spacing, DensityMode, DENSITY_COMFORTABLE_CLASS, DENSITY_COMPACT_CLASS};
