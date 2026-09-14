//! Page shell layout, navigation chrome, header, and breadcrumb rendering.

use crate::web::assets::DensityMode;

use super::primitives::{escape, flash_banner};

/// Title, one line of context, and the actions that apply to the page.
pub fn page_head(title: &str, subtitle: &str, actions: &str) -> String {
    let subtitle = if subtitle.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="sub">{}</div>"#, escape(subtitle))
    };
    let actions = if actions.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<div class="toolbar">{actions}</div>"#)
    };
    format!(
        r#"<div class="page-head"><div><h1>{title}</h1>{subtitle}</div>{actions}</div>"#,
        title = escape(title),
        subtitle = subtitle,
        actions = actions,
    )
}

/// A trail back to the list, so a detail or edit page is never a dead end. An
/// empty href renders the current location as plain text.
pub fn breadcrumb(items: &[(&str, &str)]) -> String {
    let parts = items
        .iter()
        .map(|(label, href)| {
            if href.is_empty() {
                format!("<span>{}</span>", escape(label))
            } else {
                format!(r#"<a href="{}">{}</a>"#, escape(href), escape(label))
            }
        })
        .collect::<Vec<_>>();
    format!(
        r#"<nav class="breadcrumb" aria-label="Breadcrumb">{}</nav>"#,
        parts.join(r#"<span class="sep">/</span>"#)
    )
}

/// The page shell: grouped sidebar navigation, a header, and the page body.
/// The sidebar comes from [`super::super::nav`], so a new page appears everywhere once
/// its route exists.
///
/// Renders at the comfortable density. Pages that resolve the stored density
/// preference call [`layout_with_density`] instead.
pub fn layout(title: &str, active: &str, flash: &str, body: &str) -> String {
    layout_with_density(title, active, flash, body, DensityMode::DEFAULT)
}

/// The page shell with an explicit UI density. The mode is carried as a class
/// on `<body>`, so the stylesheet's density modifiers scope the page body while
/// the chrome (sidebar, header) stays invariant.
pub fn layout_with_density(
    title: &str,
    active: &str,
    flash: &str,
    body: &str,
    density: DensityMode,
) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title} · NeoNexus</title>
<style>{css}</style>
</head>
<body class="{density_class}">
<a class="skip-link" href="#main-content">Skip to main content</a>
<header class="aws-top-bar" role="banner">
  <div class="aws-top-bar-left">
    <a class="aws-console-brand" href="/" title="Fleet overview">
      <span class="aws-brand-hex" aria-hidden="true">⬡</span>
      <span class="aws-brand-name">NeoNexus</span>
    </a>
    <div class="aws-service-menu-wrap">
      <button type="button" class="btn small" style="background: rgba(255,255,255,0.06); border: 1px solid var(--line); font-size: 11px; padding: 3px 8px; color: #fff; cursor: pointer;">Go to ▾</button>
      <div class="aws-service-menu" role="menu">{service_menu}</div>
    </div>
  </div>
  <div class="aws-top-bar-center">
    <form method="get" action="/nodes" class="aws-global-search" role="search">
      <span class="aws-search-icon" aria-hidden="true">🔍</span>
      <input type="text" id="global-resource-search" name="q" placeholder="Search nodes by name, id, client or network (/ to focus)" autocomplete="off">
      <span class="aws-search-kbd">/</span>
    </form>
  </div>
  <div class="aws-top-bar-right">
    <form method="post" action="/logout" style="margin: 0;">
      <button class="nav-item logout" type="submit" title="Sign out" style="padding: 4px 10px; font-size: 11px;">Sign out</button>
    </form>
  </div>
</header>
<header class="mobile-nav">
<details>
<summary><span class="brand mobile-brand">NeoNexus</span><span class="menu-label">Menu</span></summary>
<div class="mobile-nav-menu">
<nav aria-label="Mobile navigation">{mobile_nav}</nav>
<div class="mobile-utilities"><form method="post" action="/logout"><button class="nav-item logout" type="submit">Sign out</button></form></div>
</div>
</details>
</header>
<div class="shell">
<aside class="sidebar">
<a class="brand" href="/">NeoNexus</a>
<nav class="sidebar-nav" aria-label="Primary navigation">{nav}</nav>
<div class="sidebar-utilities"><form method="post" action="/logout"><button class="nav-item logout" type="submit">Sign out</button></form></div>
</aside>
<main class="content" id="main-content" tabindex="-1">
<div class="workspace-bar"><span><i aria-hidden="true"></i>Local operator workspace</span><a href="/metrics">Resource metrics</a></div>
{flash_banner}
{body}
</main>
</div>
<script>{script}</script>
</body>
</html>"##,
        title = escape(title),
        css = crate::web::assets::CSS,
        script = crate::web::assets::SCRIPT,
        nav = crate::web::nav::render(active),
        service_menu = crate::web::nav::service_menu(),
        mobile_nav = crate::web::nav::render(active),
        flash_banner = flash_banner(flash),
        body = body,
        density_class = density.body_class(),
    )
}
