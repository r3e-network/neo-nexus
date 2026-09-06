//! Browser assets embedded in the Rust binary.
//!
//! NeoNexus deliberately has no frontend build toolchain or static asset
//! directory. Keeping the stylesheet and progressive enhancement script in
//! focused Rust modules preserves that deployment property without leaving one
//! oversized source file to own the whole interface.

mod script;
mod styles;

pub use script::SCRIPT;
pub use styles::CSS;
