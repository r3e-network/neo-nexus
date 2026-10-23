//! Reset, typography, the shell and its navigation, panels, tables and density.
//!
//! One slice of the console stylesheet. Slices are concatenated in source
//! order by `super::css`; see `style.rs` for why the sheet is not one file.

pub(super) const RULES: &str = r#"
:root {
  color-scheme: dark;
  --bg: #090d0f;
  --bg-subtle: #0c1215;
  --panel: #10171b;
  --panel-2: #141d22;
  --panel-3: #182329;
  --panel-hover: #1c2a32;
  --glass: rgba(16, 23, 27, 0.85);
  --line: rgba(255, 255, 255, 0.08);
  --line-strong: rgba(255, 255, 255, 0.14);
  --line-glow: rgba(59, 209, 132, 0.35);

  --text: #f1f7f4;
  --muted: #9ab0a6;
  --faint: #62776e;

  --jade: #3bd184;
  --jade-strong: #54e79b;
  --jade-dim: #258c57;
  --jade-surface: rgba(59, 209, 132, 0.10);
  --jade-border: rgba(59, 209, 132, 0.32);
  --jade-glow: rgba(59, 209, 132, 0.22);
  --jade-ink: #04140b;

  --cyan: #38bdf8;
  --cyan-surface: rgba(56, 189, 248, 0.10);
  --cyan-border: rgba(56, 189, 248, 0.32);

  --amber: #fbbf24;
  --amber-surface: rgba(251, 191, 36, 0.10);
  --amber-border: rgba(251, 191, 36, 0.32);

  --red: #f43f5e;
  --red-surface: rgba(244, 63, 94, 0.10);
  --red-border: rgba(244, 63, 94, 0.32);

  --idle: #718096;

  --s1: 4px; --s2: 8px; --s3: 12px; --s4: 16px; --s5: 24px; --s6: 32px;
  --r1: 6px; --r2: 10px; --r3: 14px;

  --shadow-sm: 0 1px 3px rgba(0, 0, 0, 0.35), 0 1px 2px rgba(0, 0, 0, 0.25);
  --shadow-md: 0 4px 18px rgba(0, 0, 0, 0.45), 0 2px 4px rgba(0, 0, 0, 0.25);
  --shadow-lg: 0 12px 36px rgba(0, 0, 0, 0.55), 0 4px 10px rgba(0, 0, 0, 0.35);

  --sans: "Inter", "Segoe UI Variable", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif;
  --mono: "JetBrains Mono", ui-monospace, "Cascadia Mono", "SF Mono", Consolas, monospace;
}

* { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }

body {
  margin: 0;
  background-color: var(--bg);
  background-image:
    radial-gradient(ellipse 65% 50% at 16% -10%, rgba(59, 209, 132, 0.07) 0%, transparent 60%),
    radial-gradient(ellipse 55% 45% at 90% 100%, rgba(56, 189, 248, 0.04) 0%, transparent 60%);
  background-attachment: fixed;
  color: var(--text);
  font: 14px/1.55 var(--sans);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

/* Custom Scrollbars */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track { background: transparent; }
::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.14); border-radius: 3px; }
::-webkit-scrollbar-thumb:hover { background: rgba(255, 255, 255, 0.26); }

.skip-link {
  position: fixed; z-index: 100; left: var(--s3); top: var(--s3); padding: 8px 14px;
  transform: translateY(-180%); border: 1px solid var(--jade); border-radius: var(--r1);
  background: var(--panel); color: var(--text); font-weight: 600; box-shadow: var(--shadow-md);
}
.skip-link:focus { transform: translateY(0); }

h1 {
  margin: 0 0 var(--s4);
  font-size: clamp(23px, 2.2vw, 30px);
  line-height: 1.22;
  font-weight: 700;
  letter-spacing: -.03em;
  color: #fff;
}
.page-head h1 { margin-bottom: 0; }

h2 {
  margin: var(--s6) 0 var(--s3);
  color: #fff;
  font-size: 14px;
  font-weight: 650;
  letter-spacing: .02em;
}
h3 { margin: 0 0 var(--s3); font-size: 14px; font-weight: 650; color: var(--text); }
p { margin: 0 0 var(--s4); }

a {
  color: var(--text);
  text-decoration: underline;
  text-decoration-color: var(--line-strong);
  text-underline-offset: 3px;
  transition: color .15s ease, text-decoration-color .15s ease;
}
a:hover { color: var(--jade); text-decoration-color: var(--jade); }

:focus-visible { outline: 2px solid var(--jade); outline-offset: 3px; border-radius: var(--r1); }
code, .mono, .num, .path { font-family: var(--mono); font-size: 12.5px; }
.num, td.num, .stat-value { font-variant-numeric: tabular-nums; }

/* Shell & Navigation */
.shell { display: flex; min-height: 100vh; }

.sidebar {
  position: sticky; top: 0; width: 252px; height: 100vh; flex: 0 0 252px;
  overflow-y: auto;
  background: rgba(10, 15, 18, 0.92);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-right: 1px solid var(--line);
  padding: 22px 14px 16px;
  display: flex; flex-direction: column; gap: 2px;
}

.brand {
  position: relative;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 4px 10px 24px;
  color: #fff;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: -.01em;
  text-decoration: none;
}
.brand::before {
  content: "N";
  width: 26px; height: 26px;
  display: grid; place-items: center;
  border: 1px solid var(--jade-border);
  border-radius: 7px;
  background: linear-gradient(135deg, rgba(59, 209, 132, 0.2) 0%, rgba(59, 209, 132, 0.05) 100%);
  color: var(--jade-strong);
  font: 700 13px/1 var(--mono);
  box-shadow: 0 0 12px var(--jade-glow);
}

.nav-group { margin-bottom: var(--s3); }
.sidebar-nav { min-height: 0; display: flex; flex: 1; flex-direction: column; }
.nav-group.utility { margin-top: auto; padding-top: var(--s3); border-top: 1px solid var(--line); }

.nav-title {
  padding: 10px 10px 6px;
  color: var(--faint);
  font-size: 10.5px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: .12em;
}

.nav-item {
  display: flex; align-items: center; gap: 11px; width: 100%; min-height: 38px;
  padding: 8px 11px; border: 0; border-left: 2px solid transparent;
  border-radius: var(--r1); background: transparent; color: var(--muted); text-align: left;
  text-decoration: none; font: inherit; font-size: 13px; font-weight: 500; cursor: pointer;
  transition: all .16s ease;
}
.nav-item:hover {
  background: rgba(255, 255, 255, 0.04);
  color: #fff;
  transform: translateX(2px);
}
.nav-item.current {
  border-left-color: var(--jade);
  background: linear-gradient(90deg, rgba(59, 209, 132, 0.14) 0%, rgba(59, 209, 132, 0.03) 100%);
  color: #fff;
  font-weight: 600;
  box-shadow: inset 3px 0 10px -2px var(--jade-glow);
}

.nav-icon {
  width: 16px; height: 16px; flex: 0 0 16px;
  color: var(--faint);
  transition: color .15s ease;
}
.nav-item.current .nav-icon, .nav-item:hover .nav-icon { color: var(--jade); }

.sidebar-utilities { margin-top: var(--s2); padding-top: var(--s2); border-top: 1px solid var(--line); }
.sidebar-utilities form, .mobile-utilities form { margin: 0; }
.logout { margin-top: 2px; color: var(--faint); }
.logout:hover { color: var(--red); background: var(--red-surface); }

.content {
  width: 100%; max-width: 1600px; min-width: 0;
  padding: 36px clamp(24px, 4vw, 64px) 60px;
}

/* Mobile Nav */
.mobile-nav { display: none; }
.mobile-nav summary {
  list-style: none; min-height: 48px; display: flex; align-items: center;
  justify-content: space-between; cursor: pointer;
}
.mobile-nav summary::-webkit-details-marker { display: none; }
.mobile-nav-menu { border-top: 1px solid var(--line); padding: var(--s3); }
.mobile-nav details[open] .mobile-nav-menu { max-height: calc(100vh - 61px); overflow-y: auto; }
.mobile-utilities { margin-top: var(--s2); padding-top: var(--s2); border-top: 1px solid var(--line); }
.menu-label {
  color: var(--muted); font-size: 12px; font-weight: 650; letter-spacing: .06em;
  text-transform: uppercase;
}

/* Workspace Bar & Ambient Beacon */
.workspace-bar {
  min-height: 36px; display: flex; align-items: center; justify-content: space-between;
  gap: var(--s3); margin: -10px 0 var(--s5); padding-bottom: var(--s3); border-bottom: 1px solid var(--line);
  color: var(--faint); font-size: 11.5px;
}
.workspace-bar span { display: inline-flex; align-items: center; gap: 9px; }
.workspace-bar i {
  width: 7px; height: 7px; border-radius: 50%;
  background: var(--jade);
  box-shadow: 0 0 8px var(--jade);
  animation: pulse-beacon 2.4s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.workspace-bar a { color: var(--muted); }

@keyframes pulse-beacon {
  0%, 100% { opacity: 1; transform: scale(1); }
  50% { opacity: 0.4; transform: scale(0.85); }
}

/* Page Headers & Toolbars */
.breadcrumb { margin-bottom: var(--s2); color: var(--muted); font-size: 12.5px; }
.breadcrumb a { color: var(--muted); }
.breadcrumb .sep { margin: 0 var(--s1); color: var(--faint); }

.page-head {
  display: flex; align-items: flex-start; justify-content: space-between; gap: var(--s4);
  flex-wrap: wrap; margin-bottom: var(--s5); padding-bottom: var(--s5); border-bottom: 1px solid var(--line);
}
.page-head .sub { max-width: 780px; margin-top: 6px; color: var(--muted); font-size: 13.5px; }
.toolbar, .actions, .row-actions { display: flex; align-items: center; gap: var(--s2); flex-wrap: wrap; }
.row-actions { justify-content: flex-end; white-space: nowrap; }
.node-controls { margin-bottom: 20px; }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: var(--s3); margin-bottom: var(--s4); }
.section-head h2 { margin: 0; }

/* Surfaces, Panels & Glassmorphism */
.surface, .panel {
  background: var(--panel);
  border: 1px solid var(--line);
  border-radius: var(--r2);
  padding: var(--s5);
  box-shadow: var(--shadow-sm);
  transition: border-color .18s ease, box-shadow .18s ease;
}
.surface:hover, .panel:hover {
  border-color: rgba(255, 255, 255, 0.12);
}
.surface > :first-child, .panel > :first-child { margin-top: 0; }
.surface > :last-child, .panel > :last-child { margin-bottom: 0; }

/* Statistics Cards */
.cards, .stat-grid {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  margin: 0 0 var(--s5);
  border: 1px solid var(--line);
  border-radius: var(--r2);
  background: var(--panel);
  box-shadow: var(--shadow-sm);
  overflow: hidden;
}
.card, .stat {
  min-width: 0; padding: 18px var(--s4);
  border-right: 1px solid var(--line);
  background: transparent;
  transition: background .18s ease;
}
.card:hover, .stat:hover {
  background: rgba(255, 255, 255, 0.02);
}
.card:last-child, .stat:last-child { border-right: 0; }
.card .num, .stat-value {
  color: #fff; font-size: 25px; font-weight: 700; line-height: 1.15;
  letter-spacing: -.02em;
}
.card .lbl, .stat-label {
  margin-top: 6px; color: var(--muted); font-size: 10.5px;
  font-weight: 700; letter-spacing: .09em; text-transform: uppercase;
}
.stat-detail { margin-top: 4px; color: var(--faint); font-size: 11.5px; }
.stat.positive .stat-value { color: var(--jade-strong); }
.stat.info .stat-value { color: var(--cyan); }
.stat.warning .stat-value { color: var(--amber); }
.stat.danger .stat-value { color: var(--red); }

/* Tables */
.scroll-x {
  overflow-x: auto;
  border: 1px solid var(--line);
  border-radius: var(--r2);
  box-shadow: var(--shadow-sm);
  background: var(--panel);
}
.scroll-x table { min-width: 640px; border: 0; }
table { width: 100%; border-collapse: collapse; border: 1px solid var(--line); background: var(--panel); }
th, td { padding: 12px var(--s3); border-bottom: 1px solid var(--line); text-align: left; vertical-align: middle; }
th {
  color: var(--faint); background: rgba(12, 17, 21, 0.95);
  font-size: 10.5px; font-weight: 700;
  letter-spacing: .08em; text-transform: uppercase; white-space: nowrap;
}
tbody tr:last-child td { border-bottom: 0; }
tbody tr { transition: background .14s ease; }
tbody tr:hover td { background: rgba(255, 255, 255, 0.03); }
td.num, th.num { text-align: right; }
td.path, td.mono { color: var(--muted); word-break: break-all; }
.node-name { display: block; font-weight: 650; text-decoration: none; color: #fff; }
.node-meta { display: block; margin-top: 2px; color: var(--faint); font-size: 11.5px; }
.dashboard-table td { height: 58px; }

/* Density */
.density-comfortable { --xs: var(--s2); --sm: var(--s3); --row-pad-y: 11px; }
.density-compact { --xs: var(--s1); --sm: var(--s2); --row-pad-y: 8px; }
.density-compact td, .density-compact th { padding-top: var(--row-pad-y); padding-bottom: var(--row-pad-y); }
.density-compact .card, .density-compact .stat { padding-top: var(--sm); padding-bottom: var(--sm); }
.node-line { display: flex; align-items: center; gap: var(--sm, var(--s3)); min-height: 22px; }
.node-line .node-name { font-weight: 650; text-decoration: none; }
.node-line .node-port { margin-left: auto; color: var(--muted); }

"#;
