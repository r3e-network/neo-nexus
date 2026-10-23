//! The console stylesheet, assembled from its parts.
//!
//! NeoNexus has no asset pipeline and no static directory: whatever the browser
//! receives for `style-src` is baked into the binary. That rules out a `.css`
//! file — `--source-purity` keeps frontend sources out of a pure Rust tree,
//! which is the point of the rule and not an obstacle to route around.
//!
//! So the sheet lives in Rust string literals, split across four modules by what
//! each part is for. A single literal was 900-plus lines: long enough that
//! finding a rule meant scrolling rather than searching, and long enough that
//! every edit to it produced a diff nobody could read.
//!
//! The slices are joined in source order with nothing between them, so the
//! result is byte-for-byte the stylesheet they were authored as. That matters
//! because `security_headers` pins `style-src` to the SHA-256 of this string: a
//! separator slipped between two slices would change the digest, the browser
//! would refuse the stylesheet, and every page would render unstyled without
//! anything failing loudly enough to say why.
//!
//! For the same reason the parts are LF-only, pinned by `.gitattributes`
//! (`text eol=lf`). `cargo fmt` rewrites a file's line endings wholesale,
//! including those inside string literals, and `core.autocrlf=true` would
//! rewrite them again on checkout — git cannot tell a literal from source.
//! The digest is computed from whatever this returns, so neither would break
//! `style-src`; what it would break is reproducibility, with one commit
//! producing a different stylesheet on every machine. Pinning costs one line.

mod aws;
mod controls;
mod foundation;
mod screens;

use std::sync::OnceLock;

/// The whole sheet, joined once on first use.
///
/// Joined at runtime rather than with `concat!` because that macro only accepts
/// literals, and the slices are named constants. The join happens once; callers
/// get `&'static str` and pay a single atomic load.
pub fn css() -> &'static str {
    static SHEET: OnceLock<String> = OnceLock::new();
    SHEET.get_or_init(|| {
        [
            foundation::RULES,
            controls::RULES,
            screens::RULES,
            aws::RULES,
        ]
        .concat()
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/web/style/tests.rs"]
mod tests;
