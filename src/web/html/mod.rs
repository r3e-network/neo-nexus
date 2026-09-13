//! HTML building blocks: escaping, the shared page shell, and the small
//! component helpers the pages compose. All interpolated values flow through
//! [`escape`] so operator data can never inject markup.

mod forms;
mod page;
mod primitives;

pub use forms::*;
pub use page::*;
pub use primitives::*;

#[cfg(test)]
#[path = "../../../tests/unit/web/html/tests.rs"]
mod tests;
