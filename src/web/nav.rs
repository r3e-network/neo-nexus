//! The sidebar model. The desktop shell carried fifteen destinations; the web
//! workbench keeps all of them, grouped so the sidebar stays scannable. Each
//! entry names the page that owns it, which is also how [`render`] marks the
//! current one.

pub struct Destination {
    pub key: &'static str,
    pub href: &'static str,
    pub label: &'static str,
}

pub struct Section {
    pub title: &'static str,
    pub destinations: &'static [Destination],
}

const SECTIONS: &[Section] = &[
    Section {
        title: "Fleet",
        destinations: &[
            Destination {
                key: "home",
                href: "/",
                label: "Home",
            },
            Destination {
                key: "nodes",
                href: "/nodes",
                label: "Nodes",
            },
            Destination {
                key: "monitor",
                href: "/monitor",
                label: "Monitor",
            },
            Destination {
                key: "logs",
                href: "/logs",
                label: "Logs",
            },
        ],
    },
    Section {
        title: "Operations",
        destinations: &[
            Destination {
                key: "operations",
                href: "/operations",
                label: "Readiness",
            },
            Destination {
                key: "alerts",
                href: "/alerts",
                label: "Alerts",
            },
            Destination {
                key: "federation",
                href: "/federation",
                label: "Federation",
            },
            Destination {
                key: "roles",
                href: "/roles",
                label: "Private network",
            },
        ],
    },
    Section {
        title: "Assets",
        destinations: &[
            Destination {
                key: "runtimes",
                href: "/runtimes",
                label: "Runtimes",
            },
            Destination {
                key: "plugins",
                href: "/plugins",
                label: "Plugins",
            },
            Destination {
                key: "snapshots",
                href: "/snapshots",
                label: "Snapshots",
            },
            Destination {
                key: "wallets",
                href: "/wallets",
                label: "Wallets",
            },
            Destination {
                key: "config",
                href: "/config",
                label: "Config",
            },
        ],
    },
    Section {
        title: "Insights",
        destinations: &[
            Destination {
                key: "metrics",
                href: "/metrics",
                label: "Metrics",
            },
            Destination {
                key: "settings",
                href: "/settings",
                label: "Settings",
            },
        ],
    },
];

/// All known destination keys. The router's own test walks this list, so a page
/// can be added to the sidebar and picked up without editing two places.
pub fn keys() -> Vec<&'static str> {
    SECTIONS
        .iter()
        .flat_map(|section| section.destinations.iter().map(|d| d.key))
        .collect()
}

/// The path a destination points at. The end-to-end suite requests these, so a
/// sidebar entry that points nowhere fails a test rather than a user.
pub fn href_for(key: &str) -> Option<&'static str> {
    SECTIONS
        .iter()
        .flat_map(|section| section.destinations.iter())
        .find(|destination| destination.key == key)
        .map(|destination| destination.href)
}

/// The sidebar markup. `active` matches one [`Destination::key`]; unknown keys
/// simply leave nothing highlighted.
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
                        r#"<a class="nav-item{current}" href="{href}"{aria}>{label}</a>"#,
                        href = destination.href,
                        label = destination.label,
                    )
                })
                .collect::<String>();
            format!(
                r#"<div class="nav-group"><div class="nav-title">{}</div>{items}</div>"#,
                section.title
            )
        })
        .collect()
}

#[cfg(test)]
#[path = "../../tests/unit/web/nav/tests.rs"]
mod tests;
