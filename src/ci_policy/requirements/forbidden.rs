use super::ForbiddenCiMarker;

pub(in crate::ci_policy) const FORBIDDEN_CI_MARKERS: [ForbiddenCiMarker; 13] = [
    ForbiddenCiMarker {
        marker: "actions/setup-node",
        message: "Node setup actions do not belong in pure Rust native CI",
    },
    ForbiddenCiMarker {
        marker: "node-version:",
        message: "Node version pinning is outside the native Rust CI boundary",
    },
    ForbiddenCiMarker {
        marker: "npm run",
        message: "npm scripts would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: " npm ",
        message: "npm commands would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: "yarn ",
        message: "yarn commands would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: "pnpm ",
        message: "pnpm commands would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: "bun ",
        message: "bun commands would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: "npx ",
        message: "npx commands would reintroduce frontend or Node workflow dependencies",
    },
    ForbiddenCiMarker {
        marker: "vite ",
        message: "Vite belongs to frontend tooling, not the native Rust CI path",
    },
    ForbiddenCiMarker {
        marker: "next ",
        message: "Next.js belongs to frontend tooling, not the native Rust CI path",
    },
    ForbiddenCiMarker {
        marker: "playwright",
        message: "Browser automation should not be required for a pure native Rust app gate",
    },
    ForbiddenCiMarker {
        marker: "tauri",
        message: "Tauri/webview packaging would violate the pure native application boundary",
    },
    ForbiddenCiMarker {
        marker: "webview",
        message: "WebView packaging would violate the pure native application boundary",
    },
];
