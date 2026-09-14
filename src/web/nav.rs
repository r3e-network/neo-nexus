//! The operations-console information architecture — **the** one, singular.
//!
//! There were two navigations with different labels and different coverage: this
//! sidebar, and a hand-written `Services ▾` menu in the top bar. "Runtimes" was
//! also "AMIs & Runtimes"; "Configuration" was also "CloudFormation & Config";
//! "Signer" was also "KMS Key Management". The menu omitted Logs, Readiness,
//! Federation, Duties, Wallets, Plugins, Metrics and Settings, and uniquely
//! contained `/settings/api-tokens` — which has no nav entry at all despite
//! marking Settings active.
//!
//! The menu is generated from these sections now, so there is one vocabulary
//! and one coverage. A destination added here appears in both places or in
//! neither.

pub struct Destination {
    pub key: &'static str,
    pub href: &'static str,
    pub label: &'static str,
    icon: &'static str,
}

pub struct Section {
    pub title: &'static str,
    pub destinations: &'static [Destination],
    utility: bool,
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Overview",
        utility: false,
        destinations: &[Destination {
            key: "home",
            href: "/",
            label: "Fleet overview",
            icon: "overview",
        }],
    },
    Section {
        title: "Fleet",
        utility: false,
        destinations: &[
            Destination {
                key: "nodes",
                href: "/nodes",
                label: "Nodes",
                icon: "nodes",
            },
            // "Health" named the *host* page while chain health lives on each
            // node — two different questions under one word, and the more
            // important one was not the one this linked to.
            Destination {
                key: "monitor",
                href: "/monitor",
                label: "Host health",
                icon: "health",
            },
            Destination {
                key: "logs",
                href: "/logs",
                label: "Logs",
                icon: "logs",
            },
        ],
    },
    Section {
        title: "Operations",
        utility: false,
        destinations: &[
            Destination {
                key: "operations",
                href: "/operations",
                label: "Operations",
                icon: "readiness",
            },
            Destination {
                key: "events",
                href: "/events",
                label: "Journal",
                icon: "events",
            },
            // "Alerts" promised alarms. What this page holds is the routing
            // policy and the delivery record — where journal events are sent
            // and whether they arrived.
            Destination {
                key: "alerts",
                href: "/alerts",
                label: "Alert routing",
                icon: "alerts",
            },
        ],
    },
    Section {
        title: "Network",
        utility: false,
        destinations: &[
            Destination {
                key: "federation",
                href: "/federation",
                label: "Federation",
                icon: "federation",
            },
            // This said "Private network" and opened the per-node duty
            // support matrix. There is no private-network authoring surface in
            // this build at all, so the label named a feature that does not
            // exist while hiding the one that does.
            Destination {
                key: "roles",
                href: "/roles",
                label: "Duties",
                icon: "network",
            },
        ],
    },
    Section {
        title: "Assets",
        utility: false,
        destinations: &[
            Destination {
                key: "runtimes",
                href: "/runtimes",
                label: "Runtimes",
                icon: "runtime",
            },
            // "Snapshots" beside a workspace "Backup" entry read as two names
            // for one thing. These are *inbound* fast-sync archives a node
            // syncs from; the backup is this workspace's own state going out.
            Destination {
                key: "snapshots",
                href: "/snapshots",
                label: "Fast-sync archives",
                icon: "snapshot",
            },
            Destination {
                key: "plugins",
                href: "/plugins",
                label: "Plugins",
                icon: "plugin",
            },
            Destination {
                key: "config",
                href: "/config",
                label: "Configuration",
                icon: "config",
            },
            Destination {
                key: "wallets",
                href: "/wallets",
                label: "Wallets",
                icon: "wallet",
            },
        ],
    },
    Section {
        title: "Security",
        utility: false,
        destinations: &[
            // One binding wore five names across the console — "IAM Signer
            // Lease", "IAM Signer Role", "IAM Instance Profile & Signer
            // Identity", "Node signer", and a breadcrumb reading "KMS /
            // Customer managed keys". Two words survive: **key** on the custody
            // side, which is what this page holds, and **signer binding** on
            // the node side.
            Destination {
                key: "signer",
                href: "/signer",
                label: "Signing keys",
                icon: "signer",
            },
            Destination {
                key: "api-tokens",
                href: "/settings/api-tokens",
                label: "API tokens",
                icon: "signer",
            },
        ],
    },
    Section {
        title: "Workspace",
        utility: true,
        destinations: &[
            Destination {
                key: "metrics",
                href: "/metrics",
                label: "Metrics",
                icon: "metrics",
            },
            // Registered, rendered, and reachable only by typing the URL: no
            // nav entry and no inbound link from any page. Taking a backup
            // before a risky change is not a thing an operator should have to
            // already know about.
            Destination {
                key: "backup",
                href: "/backup",
                label: "Workspace backup",
                icon: "snapshot",
            },
            Destination {
                key: "settings",
                href: "/settings",
                label: "Settings",
                icon: "settings",
            },
        ],
    },
];

/// The same destinations, as a compact jump list for the top bar.
///
/// Generated rather than written, because the hand-written one drifted: it used
/// different labels for the same pages and omitted eight of them.
pub fn service_menu() -> String {
    SECTIONS
        .iter()
        .map(|section| {
            let links = section
                .destinations
                .iter()
                .map(|destination| {
                    format!(
                        r#"<a href="{href}" class="aws-service-link"><strong>{label}</strong></a>"#,
                        href = destination.href,
                        label = destination.label,
                    )
                })
                .collect::<String>();
            format!(
                r#"<div class="aws-service-group"><div class="aws-service-group-title">{title}</div>{links}</div>"#,
                title = section.title,
            )
        })
        .collect()
}

/// Every destination as `(key, label)`, for tests that check the console says
/// the same word twice.
pub fn destinations() -> Vec<(&'static str, &'static str)> {
    SECTIONS
        .iter()
        .flat_map(|section| section.destinations.iter())
        .map(|destination| (destination.key, destination.label))
        .collect()
}

pub fn keys() -> Vec<&'static str> {
    SECTIONS
        .iter()
        .flat_map(|section| {
            section
                .destinations
                .iter()
                .map(|destination| destination.key)
        })
        .collect()
}

pub fn href_for(key: &str) -> Option<&'static str> {
    SECTIONS
        .iter()
        .flat_map(|section| section.destinations.iter())
        .find(|destination| destination.key == key)
        .map(|destination| destination.href)
}

pub fn render(active: &str) -> String {
    SECTIONS
        .iter()
        .map(|section| {
            let items = section
                .destinations
                .iter()
                .map(|destination| {
                    let (current, aria) = if destination.key == active {
                        (" current", r#" aria-current="page""#)
                    } else {
                        ("", "")
                    };
                    format!(
                        r#"<a class="nav-item{current}" href="{href}"{aria}>{icon}<span>{label}</span></a>"#,
                        href = destination.href,
                        label = destination.label,
                        icon = icon(destination.icon),
                    )
                })
                .collect::<String>();
            format!(
                r#"<div class="nav-group{utility}"><div class="nav-title">{title}</div>{items}</div>"#,
                utility = if section.utility { " utility" } else { "" },
                title = section.title,
            )
        })
        .collect()
}

fn icon(name: &str) -> &'static str {
    match name {
        // Each branch is a complete static icon so no operator text enters SVG.
        "overview" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 3h7v7H3zM14 3h7v4h-7zM14 11h7v10h-7zM3 14h7v7H3z"/></svg>"#
        }
        "nodes" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="3" width="16" height="7" rx="1"/><rect x="4" y="14" width="16" height="7" rx="1"/><path d="M8 6.5h.01M8 17.5h.01"/></svg>"#
        }
        "health" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 12h4l2-6 4 12 2-6h6"/></svg>"#
        }
        "logs" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M5 4h14v16H5zM8 8h8M8 12h8M8 16h5"/></svg>"#
        }
        "readiness" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 12l5 5L20 6"/></svg>"#
        }
        "events" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M5 5h14v14H5zM8 9h8M8 13h5M8 17h3"/></svg>"#
        }
        "alerts" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3a6 6 0 0 0-6 6v4l-2 3h16l-2-3V9a6 6 0 0 0-6-6zM10 20h4"/></svg>"#
        }
        "federation" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="6" cy="12" r="3"/><circle cx="18" cy="6" r="3"/><circle cx="18" cy="18" r="3"/><path d="M9 11l6-4M9 13l6 4"/></svg>"#
        }
        "network" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="9"/><path d="M3 12h18M12 3c3 3 3 15 0 18M12 3c-3 3-3 15 0 18"/></svg>"#
        }
        "runtime" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M5 4h14v16H5zM8 8l3 3-3 3M13 16h3"/></svg>"#
        }
        "snapshot" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 7h16v13H4zM8 4h8v3M8 12h8"/></svg>"#
        }
        "plugin" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M8 3v5H3v8h5v5h8v-5h5V8h-5V3z"/></svg>"#
        }
        "config" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 7h10M18 7h2M4 12h2M10 12h10M4 17h7M15 17h5"/><circle cx="16" cy="7" r="2"/><circle cx="8" cy="12" r="2"/><circle cx="13" cy="17" r="2"/></svg>"#
        }
        "wallet" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 6h15v13H4zM4 9h15M15 13h6v4h-6z"/></svg>"#
        }
        "signer" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 3l7 3v5c0 5-3 8-7 10-4-2-7-5-7-10V6zM9 12l2 2 4-5"/></svg>"#
        }
        "metrics" => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 19V9M10 19V5M16 19v-7M22 19V3"/></svg>"#
        }
        _ => {
            r#"<svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="8"/><path d="M12 8v8M8 12h8"/></svg>"#
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/web/nav/tests.rs"]
mod tests;
