use std::{collections::BTreeMap, collections::BTreeSet, path::Path};

use anyhow::Result;

use super::{
    files::AuditFile,
    rules::{
        cargo_dependency_present, cargo_dependency_requirements, forbidden_markers,
        required_marker_path_label, required_markers,
    },
    NativeUiAuditFinding,
};

pub(super) fn required_findings(
    root: &Path,
    files: &[AuditFile],
) -> Result<Vec<NativeUiAuditFinding>> {
    let file_text = files
        .iter()
        .map(|file| (file.relative_path.as_str(), file.text.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut findings = Vec::new();

    for requirement in cargo_dependency_requirements() {
        if !cargo_dependency_present(root, requirement.marker)? {
            findings.push(NativeUiAuditFinding {
                path: "Cargo.toml".to_string(),
                marker: requirement.marker.to_string(),
                category: "missing-required-ui-marker".to_string(),
                message: requirement.message.to_string(),
            });
        }
    }

    for requirement in required_markers() {
        let present = std::iter::once(requirement.path)
            .chain(requirement.alternate_paths.iter().copied())
            .any(|path| {
                file_text
                    .get(path)
                    .is_some_and(|text| text.contains(requirement.marker))
            });
        if !present {
            findings.push(NativeUiAuditFinding {
                path: required_marker_path_label(&requirement),
                marker: requirement.marker.to_string(),
                category: "missing-required-ui-marker".to_string(),
                message: requirement.message.to_string(),
            });
        }
    }

    Ok(findings)
}

pub(super) fn forbidden_findings(files: &[AuditFile]) -> Vec<NativeUiAuditFinding> {
    let markers = forbidden_markers();
    let mut findings = Vec::new();
    let mut seen = BTreeSet::new();

    for file in files
        .iter()
        .filter(|file| file.relative_path.starts_with("src/"))
    {
        for marker in &markers {
            if file.text.contains(&marker.marker)
                && seen.insert((file.relative_path.clone(), marker.marker.clone()))
            {
                findings.push(NativeUiAuditFinding {
                    path: file.relative_path.clone(),
                    marker: marker.marker.clone(),
                    category: "forbidden-ui-marker".to_string(),
                    message: marker.message.to_string(),
                });
            }
        }
    }

    findings
}
