mod directories;
mod files;
mod webview;

use super::SourcePurityFinding;

use self::{
    directories::{disallowed_directory_rule, should_skip_directory_rule},
    files::disallowed_file_rule,
    webview::is_webview_cargo_package_name,
};

pub(super) fn should_skip_directory(name: &str) -> bool {
    should_skip_directory_rule(name)
}

pub(super) fn disallowed_directory(name: &str, path: String) -> Option<SourcePurityFinding> {
    disallowed_directory_rule(name, path)
}

pub(super) fn disallowed_file(name: &str, path: String) -> Option<SourcePurityFinding> {
    disallowed_file_rule(name, path)
}

pub(super) fn is_webview_cargo_package(name: &str) -> bool {
    is_webview_cargo_package_name(name)
}
