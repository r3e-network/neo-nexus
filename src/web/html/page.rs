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
    <a class="aws-console-brand" href="/" title="NeoNexus Cloud Control Plane">
      <span class="aws-brand-hex" aria-hidden="true">⬡</span>
      <span class="aws-brand-name">NeoNexus</span>
      <span class="aws-service-label">EC2 Control Plane</span>
    </a>
    <div class="aws-service-menu-wrap">
      <button type="button" class="btn small" style="background: rgba(255,255,255,0.06); border: 1px solid var(--line); font-size: 11px; padding: 3px 8px; color: #fff; cursor: pointer;">Services ▾</button>
      <div class="aws-service-menu" role="menu">
        <div class="aws-service-group">
          <div class="aws-service-group-title">Compute</div>
          <a href="/nodes" class="aws-service-link"><span>⬡</span> <strong>EC2 Instances</strong></a>
          <a href="/nodes/new" class="aws-service-link"><span>➕</span> <strong>Launch Instance</strong></a>
          <a href="/runtimes" class="aws-service-link"><span>💿</span> <strong>AMIs &amp; Runtimes</strong></a>
        </div>
        <div class="aws-service-group">
          <div class="aws-service-group-title">Management &amp; Governance</div>
          <a href="/monitor" class="aws-service-link"><span>📊</span> <strong>CloudWatch Metrics</strong></a>
          <a href="/alerts" class="aws-service-link"><span>🚨</span> <strong>CloudWatch Alarms &amp; SNS</strong></a>
          <a href="/operations" class="aws-service-link"><span>⚙️</span> <strong>Systems Manager OpsCenter</strong></a>
          <a href="/events" class="aws-service-link"><span>📜</span> <strong>CloudTrail Event History</strong></a>
          <a href="/config" class="aws-service-link"><span>🏗️</span> <strong>CloudFormation &amp; Config</strong></a>
        </div>
        <div class="aws-service-group">
          <div class="aws-service-group-title">Storage &amp; Security</div>
          <a href="/snapshots" class="aws-service-link"><span>💾</span> <strong>EBS Snapshots</strong></a>
          <a href="/signer" class="aws-service-link"><span>🔒</span> <strong>KMS Key Management</strong></a>
          <a href="/settings/api-tokens" class="aws-service-link"><span>🔑</span> <strong>IAM API Credentials</strong></a>
        </div>
      </div>
    </div>
  </div>
  <div class="aws-top-bar-center">
    <form method="get" action="/nodes" class="aws-global-search" role="search">
      <span class="aws-search-icon" aria-hidden="true">🔍</span>
      <input type="text" id="global-resource-search" name="q" placeholder="Search instances, services, logs, or templates (/ to focus)" autocomplete="off">
      <span class="aws-search-kbd">/</span>
    </form>
  </div>
  <div class="aws-top-bar-right">
    <a href="/operations" class="aws-nav-action" title="Hermes Autonomous AI Copilot &amp; Systems Manager">
      <span class="aws-copilot-pill">✦ Hermes AI</span>
    </a>
    <a href="/monitor" class="aws-nav-action" title="CloudWatch Health: All Systems Operational">
      <span class="status-dot running" style="width: 8px; height: 8px;"></span>
      <span class="aws-header-text">Health 2/2</span>
    </a>
    <div class="aws-region-pill" title="Active Cloud Region">
      <span class="aws-region-dot"></span>
      <span class="aws-header-text">neo:mesh-1a</span>
    </div>
    <div class="aws-account-badge" title="Authenticated IAM Role">
      <span class="aws-user-icon">👤</span>
      <span class="mono" style="font-size: 11px;">arn:neo:iam::nexus:operator</span>
    </div>
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
        mobile_nav = crate::web::nav::render(active),
        flash_banner = flash_banner(flash),
        body = body,
        density_class = density.body_class(),
    )
}
