use std::{ffi::OsStr, path::Path};

#[derive(Debug, Clone, Copy)]
pub(super) struct BlockedMarker {
    name: &'static str,
    suffix: char,
    pub(super) category: &'static str,
}

impl BlockedMarker {
    pub(super) fn token(self) -> String {
        marker_token(self.name, self.suffix)
    }

    pub(super) fn is_test_assertion_shortcut(self) -> bool {
        self.category == "fallible-result-shortcut"
    }
}

pub(super) fn blocked_markers() -> [BlockedMarker; 11] {
    [
        BlockedMarker {
            name: "unwrap",
            suffix: '(',
            category: "fallible-result-shortcut",
        },
        BlockedMarker {
            name: "expect",
            suffix: '(',
            category: "fallible-result-shortcut",
        },
        BlockedMarker {
            name: "panic",
            suffix: '!',
            category: "explicit-process-panic",
        },
        BlockedMarker {
            name: "todo",
            suffix: '!',
            category: "unfinished-implementation",
        },
        BlockedMarker {
            name: "unimplemented",
            suffix: '!',
            category: "unfinished-implementation",
        },
        BlockedMarker {
            name: "dbg",
            suffix: '!',
            category: "debug-instrumentation",
        },
        BlockedMarker {
            name: "ScrollArea",
            suffix: ':',
            category: "document-style-native-layout",
        },
        BlockedMarker {
            name: "TableBuilder",
            suffix: ':',
            category: "document-style-native-layout",
        },
        BlockedMarker {
            name: "show_rows",
            suffix: '(',
            category: "document-style-native-layout",
        },
        BlockedMarker {
            name: "vertical_scroll",
            suffix: '(',
            category: "document-style-native-layout",
        },
        BlockedMarker {
            name: "horizontal_scroll",
            suffix: '(',
            category: "document-style-native-layout",
        },
    ]
}

pub(super) fn marker_token(name: &str, suffix: char) -> String {
    let mut token = name.to_string();
    token.push(suffix);
    token
}

pub(super) fn should_skip_directory(name: &str) -> bool {
    matches!(name, ".git" | "target" | "dist")
}

pub(super) fn is_rust_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension == "rs")
}

pub(super) fn is_maintenance_text(path: &Path) -> bool {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "json" | "md" | "toml" | "yml" | "yaml"
            )
        })
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name.to_ascii_lowercase().as_str(),
                "license" | "makefile" | "notice"
            )
        })
}

pub(super) fn is_test_source(path: &Path) -> bool {
    if path
        .components()
        .any(|component| component.as_os_str() == OsStr::new("tests"))
    {
        return true;
    }

    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name.ends_with("_test.rs") || name.ends_with("_tests.rs"))
}

pub(super) fn snippet(line: &str) -> String {
    const MAX_SNIPPET_CHARS: usize = 96;
    let trimmed = line.trim();
    if trimmed.chars().count() <= MAX_SNIPPET_CHARS {
        return trimmed.to_string();
    }
    let mut preview = trimmed.chars().take(MAX_SNIPPET_CHARS).collect::<String>();
    preview.push_str("...");
    preview
}
