#[derive(Debug, Clone, Copy)]
pub(in crate::native_ui_audit) struct RequiredMarker {
    pub(in crate::native_ui_audit) path: &'static str,
    pub(in crate::native_ui_audit) alternate_paths: &'static [&'static str],
    pub(in crate::native_ui_audit) marker: &'static str,
    pub(in crate::native_ui_audit) message: &'static str,
}

#[derive(Debug, Clone)]
pub(in crate::native_ui_audit) struct ForbiddenMarker {
    pub(in crate::native_ui_audit) marker: String,
    pub(in crate::native_ui_audit) message: &'static str,
}

pub(in crate::native_ui_audit) fn required_marker_path_label(
    requirement: &RequiredMarker,
) -> String {
    if requirement.alternate_paths.is_empty() {
        requirement.path.to_string()
    } else {
        std::iter::once(requirement.path)
            .chain(requirement.alternate_paths.iter().copied())
            .collect::<Vec<_>>()
            .join(" or ")
    }
}
