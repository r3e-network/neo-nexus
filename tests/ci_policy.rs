use neo_nexus::ci_policy::CiPolicyChecker;
#[path = "ci_policy/acceptance.rs"]
mod acceptance;
#[path = "ci_policy/contract.rs"]
mod contract;
#[path = "ci_policy/doc_links.rs"]
mod doc_links;
#[path = "ci_policy/orphan_modules.rs"]
mod orphan_modules;
#[path = "ci_policy/rejections.rs"]
mod rejections;
#[path = "ci_policy/source_quality.rs"]
mod source_quality;
#[path = "ci_policy/workflow_shells.rs"]
mod workflow_shells;
