//! Browser assets embedded in the Rust binary.
//!
//! NeoNexus deliberately has no frontend build toolchain or static asset
//! directory. The stylesheet is a `.css` file baked in with `include_str!`, and
//! the progressive enhancement script is a Rust module because the security
//! headers hash it as one string. Neither is a deployment artifact to copy
//! alongside the binary, and neither is an oversized source file.

mod script;
mod styles;
mod theme;

pub use script::SCRIPT;
pub use styles::CSS;
pub use theme::{spacing, DensityMode, DENSITY_COMFORTABLE_CLASS, DENSITY_COMPACT_CLASS};
