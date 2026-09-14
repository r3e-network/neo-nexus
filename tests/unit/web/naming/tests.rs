//! One name per destination, and a breadcrumb that goes somewhere.
//!
//! `/monitor` was the nav's "Health", a breadcrumb reading "CloudWatch /
//! Metrics / All metrics" whose first two crumbs both linked to `/monitor`
//! itself, and a title reading "CloudWatch Metrics & Telemetry". `/config` had
//! four names across the nav, its own breadcrumb, its own title and the
//! services menu. **The word an operator clicked never appeared on the page
//! that opened.**
//!
//! These are shape checks, not a spell-checker: a crumb must not link to the
//! page it is on, and the nav's word for a destination must appear somewhere on
//! it.

use std::{collections::BTreeMap, fs, path::Path};

fn page_sources() -> Vec<(String, String)> {
    fn walk(dir: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                if let Ok(text) = fs::read_to_string(&path) {
                    out.push((path.display().to_string(), text));
                }
            }
        }
    }
    let mut sources = Vec::new();
    walk(Path::new("src/web/pages"), &mut sources);
    assert!(!sources.is_empty(), "no page sources found");
    sources
}

/// One `breadcrumb(&[ … ])` call: the file it sits in, and its crumbs in order.
///
/// Grouped per call rather than per file, because a page module can hold more
/// than one handler and two trails sharing a first crumb is not a repeat.
fn breadcrumb_trails() -> Vec<(String, Vec<(String, String)>)> {
    let mut trails = Vec::new();
    for (path, text) in page_sources() {
        let mut rest = text.as_str();
        while let Some(at) = rest.find("breadcrumb(&[") {
            let after = &rest[at..];
            let Some(end) = after.find("]);") else { break };
            let block = &after[..end];
            let mut crumbs = Vec::new();
            for line in block.lines() {
                let line = line.trim();
                if line.starts_with("//") {
                    continue;
                }
                // `("Label", "/href"),`
                let Some(open) = line.find("(\"") else {
                    continue;
                };
                let tail = &line[open + 2..];
                let Some(label_end) = tail.find('"') else {
                    continue;
                };
                let label = &tail[..label_end];
                let after_label = &tail[label_end + 1..];
                let Some(href_open) = after_label.find('"') else {
                    continue;
                };
                let href_tail = &after_label[href_open + 1..];
                let Some(href_end) = href_tail.find('"') else {
                    continue;
                };
                crumbs.push((label.to_string(), href_tail[..href_end].to_string()));
            }
            if !crumbs.is_empty() {
                trails.push((path.clone(), crumbs));
            }
            rest = &after[end..];
        }
    }
    assert!(
        trails.len() > 10,
        "only {} breadcrumb trails parsed; the scan has drifted from the pages",
        trails.len()
    );
    trails
}

/// Every crumb, flattened, with the file it came from.
fn breadcrumb_crumbs() -> Vec<(String, String, String)> {
    breadcrumb_trails()
        .into_iter()
        .flat_map(|(file, crumbs)| {
            crumbs
                .into_iter()
                .map(move |(label, href)| (file.clone(), label, href))
        })
        .collect()
}

/// A crumb linking to the page it sits on is a control that does nothing, and
/// two of them in a row is a trail that does not lead anywhere.
#[test]
fn no_breadcrumb_repeats_the_same_destination() {
    let mut violations = Vec::new();
    for (file, crumbs) in breadcrumb_trails() {
        let mut seen: BTreeMap<&str, &str> = BTreeMap::new();
        for (label, href) in &crumbs {
            if href.is_empty() {
                continue;
            }
            if let Some(previous) = seen.insert(href.as_str(), label.as_str()) {
                violations.push(format!(
                    "{file}: {href} is linked twice, as {previous:?} and as {label:?}"
                ));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "a breadcrumb links to the same destination more than once, so the trail does not \
         lead anywhere:\n  {}",
        violations.join("\n  ")
    );
}

/// The AWS service vocabulary is not this product's.
///
/// These are not cosmetic: "IAM", "KMS" and "EBS" are genuinely distinct
/// services in AWS, so borrowing them to describe one signer binding, one key
/// store and one inbound archive teaches a model that is wrong.
#[test]
fn no_breadcrumb_borrows_another_products_service_names() {
    const BORROWED: [&str; 8] = [
        "CloudWatch",
        "CloudTrail",
        "CloudFormation",
        "Systems Manager",
        "Elastic Block Store",
        "IAM",
        "KMS",
        "EC2",
    ];
    let violations: Vec<String> = breadcrumb_crumbs()
        .into_iter()
        .filter(|(_, label, _)| BORROWED.contains(&label.as_str()))
        .map(|(file, label, _)| format!("{file}: breadcrumb crumb {label:?}"))
        .collect();
    assert!(
        violations.is_empty(),
        "a breadcrumb names another product's service. Name what the page is:\n  {}",
        violations.join("\n  ")
    );
}

/// The word an operator clicked has to appear on the page that opens.
#[test]
fn every_nav_label_appears_on_the_page_it_opens() {
    let sources = page_sources();
    let mut missing = Vec::new();
    for (key, label) in crate::web::nav::destinations() {
        // The page module is found by its nav key; a handful do not map
        // one-to-one and are checked by the route-reachability gate instead.
        let Some((_, text)) = sources
            .iter()
            .find(|(path, _)| path.contains(&format!("/{}.rs", key.replace('-', "_"))))
        else {
            continue;
        };
        if !text.contains(label) {
            missing.push(format!(
                "{key}: the nav says {label:?}, the page never does"
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "the word an operator clicked does not appear on the page that opens:\n  {}",
        missing.join("\n  ")
    );
}
