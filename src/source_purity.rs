mod cargo;
mod checker;
mod model;
mod rules;
mod scan;

#[cfg(test)]
#[path = "../tests/unit/source_purity/tests.rs"]
mod tests;

pub use self::{
    checker::SourcePurityChecker,
    model::{SourcePurityFinding, SourcePurityReport},
};
