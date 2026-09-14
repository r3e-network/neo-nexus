//! NeoNexus operations-console visual system.
//!
//! Crafted with a modern, high-density obsidian-jade design language
//! featuring glassmorphic depth, ambient neon accents, refined typography,
//! and responsive micro-interactions.

pub const CSS: &str = r#"
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

/* Status Badges & Dots */
.status-dot { width: 8px; height: 8px; flex: 0 0 8px; border-radius: 50%; background: var(--idle); }
.status-dot.running {
  background: var(--jade);
  box-shadow: 0 0 8px var(--jade);
  animation: pulse-beacon 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.status-dot.starting { background: var(--amber); box-shadow: 0 0 8px var(--amber); }
.status-dot.error { background: var(--red); box-shadow: 0 0 8px var(--red); }

.badge {
  display: inline-flex; align-items: center; gap: 6px; padding: 3px 9px;
  border: 1px solid var(--line-strong); border-radius: 999px; color: var(--muted);
  font-size: 11px; font-weight: 650; letter-spacing: .02em;
  transition: all .16s ease;
}
.badge::before { content: ""; width: 5px; height: 5px; border-radius: 50%; background: currentColor; }
.badge.running {
  color: var(--jade-strong);
  border-color: var(--jade-border);
  background: var(--jade-surface);
  box-shadow: 0 0 10px -2px var(--jade-glow);
}
.badge.running::before {
  animation: pulse-beacon 2s infinite;
}
.badge.starting {
  color: var(--amber);
  border-color: var(--amber-border);
  background: var(--amber-surface);
}
.badge.error {
  color: var(--red);
  border-color: var(--red-border);
  background: var(--red-surface);
}
.badge.stopped, .badge.unknown { color: var(--idle); }
/* Chain health, coloured by tone rather than by state name, so the nine states
   read as four weights an operator can scan. `health-neutral` is grey and never
   green: "we have not looked" must not render as a pass. */
.badge.health-good {
  color: var(--jade-strong);
  border-color: var(--jade-border);
  background: var(--jade-surface);
}
.badge.health-working {
  color: var(--cyan);
  border-color: var(--cyan-border);
  background: var(--cyan-surface);
}
.badge.health-warning {
  color: var(--amber);
  border-color: var(--amber-border);
  background: var(--amber-surface);
}
.badge.health-bad {
  color: var(--red);
  border-color: var(--red-border);
  background: var(--red-surface);
}
.badge.health-neutral { color: var(--idle); }
.badge.event-info {
  color: var(--cyan);
  border-color: var(--cyan-border);
  background: var(--cyan-surface);
}
.badge.event-warning {
  color: var(--amber);
  border-color: var(--amber-border);
  background: var(--amber-surface);
}
.badge.event-critical {
  color: var(--red);
  border-color: var(--red-border);
  background: var(--red-surface);
}
.tone-info { color: var(--cyan); }
.tone-warning { color: var(--amber); }
.tone-danger { color: var(--red); }

/* Buttons & Interactive Controls */
button, .btn, .button {
  min-height: 35px; display: inline-flex; align-items: center;
  justify-content: center; gap: 6px; padding: 7px 14px;
  border: 1px solid var(--line-strong);
  border-radius: var(--r1);
  background: rgba(255, 255, 255, 0.04);
  color: var(--text); font: inherit;
  font-size: 13px; font-weight: 550; line-height: 1.35;
  text-decoration: none; cursor: pointer;
  box-shadow: var(--shadow-sm);
  transition: all .16s cubic-bezier(0.16, 1, 0.3, 1);
}
button:hover, .btn:hover, .button:hover {
  border-color: rgba(255, 255, 255, 0.22);
  background: rgba(255, 255, 255, 0.08);
  color: #fff;
  transform: translateY(-1px);
}
button:active, .btn:active, .button:active {
  transform: translateY(1px);
}
button:disabled, .btn.disabled { opacity: .42; cursor: not-allowed; transform: none; }

button.primary, .btn.primary {
  border: 1px solid var(--jade);
  background: linear-gradient(135deg, #3bd184 0%, #2ec274 100%);
  color: var(--jade-ink); font-weight: 700;
  box-shadow: 0 2px 10px rgba(59, 209, 132, 0.25);
}
button.primary:hover, .btn.primary:hover {
  border-color: var(--jade-strong);
  background: linear-gradient(135deg, #55e398 0%, #3bd184 100%);
  box-shadow: 0 4px 16px rgba(59, 209, 132, 0.35);
  color: var(--jade-ink);
}

button.danger, .btn.danger {
  border-color: var(--red-border);
  background: var(--red-surface);
  color: #fda4af;
}
button.danger:hover, .btn.danger:hover {
  border-color: var(--red);
  background: rgba(244, 63, 94, 0.2);
  color: #fff;
  box-shadow: 0 4px 14px rgba(244, 63, 94, 0.25);
}

.btn.small, button.small { min-height: 29px; padding: 4px 10px; font-size: 12px; }

/* Forms & Inputs */
.grid {
  display: grid; grid-template-columns: repeat(auto-fit, minmax(236px, 1fr));
  gap: var(--s3) var(--s4); align-items: start;
}
.grid .span-all { grid-column: 1 / -1; }
fieldset { border: 0; padding: 0; margin: 0; }

.field {
  display: flex; flex-direction: column; gap: 6px; margin-bottom: var(--s3);
  color: var(--muted); font-size: 12px;
}
.field > span, .field label { color: var(--muted); font-weight: 650; letter-spacing: .01em; }

.field input, .field select, .field textarea {
  width: 100%; min-width: 0; padding: 9px 12px;
  border: 1px solid var(--line-strong); border-radius: var(--r1);
  background: rgba(8, 12, 15, 0.85);
  color: var(--text); font: inherit;
  transition: border-color .15s ease, box-shadow .15s ease;
}
.field input.mono, .field input[name=binary_path], .field input[name=runtime_version] {
  font-family: var(--mono); font-size: 12.5px;
}
.field input:hover, .field select:hover, .field textarea:hover {
  border-color: rgba(255, 255, 255, 0.24);
}
.field input:focus, .field select:focus, .field textarea:focus {
  border-color: var(--jade);
  outline: none;
  box-shadow: 0 0 0 3px var(--jade-surface);
}

.field .help { color: var(--faint); font-size: 11.5px; font-weight: 400; }
.field .error { color: var(--red); font-size: 12px; font-weight: 500; }
.field.invalid input, .field.invalid select { border-color: var(--red); }

.filters { display: flex; align-items: flex-end; gap: var(--s2); flex-wrap: wrap; margin-bottom: var(--s4); }
.filters .field { margin-bottom: 0; }
.filters .field input, .filters .field select { width: auto; min-width: 150px; }

.filter-check {
  min-height: 36px; display: inline-flex; align-items: center; gap: 8px;
  padding: 7px 11px; border: 1px solid var(--line-strong); border-radius: var(--r1);
  color: var(--muted); cursor: pointer; transition: all .15s ease;
}
.filter-check:hover { border-color: rgba(255, 255, 255, 0.24); color: var(--text); }
.filter-check input { width: 16px; height: 16px; margin: 0; accent-color: var(--jade); }

.form-actions {
  display: flex; align-items: center; gap: var(--s2); flex-wrap: wrap;
  margin-top: var(--s2); padding-top: var(--s4); border-top: 1px solid var(--line);
}
.form-actions .spacer { flex: 1; }

/* Flashes & Notices */
.flash, .notice {
  margin-bottom: var(--s4); padding: 12px var(--s4);
  border: 1px solid var(--line);
  border-left: 3px solid var(--cyan);
  border-radius: var(--r1);
  background: var(--panel);
  color: var(--text); font-size: 13px;
  box-shadow: var(--shadow-sm);
  animation: slide-banner .22s cubic-bezier(0.16, 1, 0.3, 1);
}
@keyframes slide-banner {
  from { opacity: 0; transform: translateY(-6px); }
  to { opacity: 1; transform: translateY(0); }
}

.notice.warn { border-left-color: var(--amber); background: rgba(251, 191, 36, 0.05); }
.notice.danger { border-left-color: var(--red); background: rgba(244, 63, 94, 0.05); }

.empty {
  padding: 52px var(--s5); border: 1px dashed var(--line-strong); border-radius: var(--r2);
  background: var(--panel); text-align: center; color: var(--muted);
}
.empty h2 { margin: 0 0 var(--s2); color: var(--text); font-size: 15px; }
.empty .actions { justify-content: center; margin-top: var(--s4); }

.muted { color: var(--muted); }
.err { color: var(--red); font-size: 13px; }
time { display: block; white-space: nowrap; font-variant-numeric: tabular-nums; }
time .elapsed { display: block; color: var(--faint); font: 11px/1.4 var(--sans); }

pre {
  overflow-x: auto; padding: var(--s4);
  border: 1px solid var(--line);
  border-radius: var(--r2);
  background: #080c0f; font: 12.5px/1.6 var(--mono);
}

.facts { max-width: 720px; }
.facts th { width: 34%; background: transparent; }
.facts td { font-family: var(--mono); font-size: 12.5px; word-break: break-all; }

/* Fleet Overview Dashboard */
.dashboard-grid {
  display: grid; grid-template-columns: minmax(0, 1fr) minmax(320px, .75fr);
  gap: var(--s4); margin-top: var(--s5);
}
.readiness-score {
  display: flex; align-items: flex-end; justify-content: space-between;
  gap: var(--s4); margin-bottom: var(--s3);
}
.readiness-score strong {
  font: 700 32px/1 var(--mono); color: var(--jade-strong);
  text-shadow: 0 0 16px var(--jade-glow);
}
.readiness-score span { color: var(--muted); font-size: 12px; }

progress { width: 100%; height: 8px; border: 0; border-radius: 4px; background: var(--panel-2); overflow: hidden; }
progress::-webkit-progress-bar { background: var(--panel-2); }
progress::-webkit-progress-value, progress::-moz-progress-bar { background: linear-gradient(90deg, #2ec274, #54e79b); box-shadow: 0 0 10px var(--jade); }

.issue-list, .activity-list { list-style: none; padding: 0; margin: var(--s4) 0 0; }
.issue-list li, .activity-list li {
  display: grid; grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--s3); padding: 11px 0; border-top: 1px solid var(--line);
  transition: background .12s ease;
}
.issue-list a, .activity-list strong { font-weight: 600; text-decoration: none; color: #fff; }
.activity-list p { margin: 3px 0 0; color: var(--muted); font-size: 12px; }
.activity-list time { color: var(--faint); font-size: 11px; }

.resource-strip {
  display: grid; grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: var(--s4); border: 1px solid var(--line); border-radius: var(--r2);
  background: var(--panel); box-shadow: var(--shadow-sm);
}
.resource { padding: 14px var(--s4); border-right: 1px solid var(--line); }
.resource:last-child { border-right: 0; }
.resource strong { display: block; font: 650 14px/1.3 var(--mono); color: #fff; }
.resource span {
  display: block; margin-top: 3px; color: var(--faint); font-size: 10px;
  letter-spacing: .08em; text-transform: uppercase;
}

/* Signer */
.subnav {
  display: flex; gap: 4px; overflow-x: auto; margin: -8px 0 var(--s5);
  border-bottom: 1px solid var(--line);
}
.subnav a {
  min-height: 40px; display: inline-flex; align-items: center; padding: 0 14px;
  border-bottom: 2px solid transparent; color: var(--muted); text-decoration: none;
  white-space: nowrap; font-weight: 500; font-size: 13px;
  transition: all .15s ease;
}
.subnav a:hover { color: #fff; }
.subnav a.current, .subnav a[aria-current=page] {
  border-bottom-color: var(--jade); color: var(--jade-strong); font-weight: 600;
}

.custody-band {
  display: grid; grid-template-columns: auto 1fr; gap: var(--s3); align-items: start;
  margin-bottom: var(--s5); padding: var(--s4); border: 1px solid var(--line);
  border-left: 3px solid var(--jade); border-radius: var(--r1); background: var(--panel);
  box-shadow: var(--shadow-sm);
}
.custody-band.warn { border-left-color: var(--amber); }
.custody-band.danger { border-left-color: var(--red); }
.custody-mark {
  width: 28px; height: 28px; display: grid; place-items: center;
  border: 1px solid currentColor; border-radius: 50%; color: var(--jade);
  font: 700 12px/1 var(--mono); box-shadow: 0 0 10px -2px currentColor;
}
.custody-band.warn .custody-mark { color: var(--amber); }
.custody-band.danger .custody-mark { color: var(--red); }
.custody-band strong { display: block; margin-bottom: 2px; color: #fff; }
.custody-band p { margin: 0; color: var(--muted); font-size: 12.5px; }

.signer-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--s4); }
.summary-list { list-style: none; padding: 0; margin: 0; }
.summary-list li {
  display: flex; justify-content: space-between; gap: var(--s3);
  padding: 10px 0; border-top: 1px solid var(--line);
}
.summary-list li:first-child { border-top: 0; padding-top: 0; }
.summary-list span { color: var(--muted); }

.secret-value {
  display: block; margin-top: var(--s2); padding: 11px;
  border: 1px solid var(--amber-border);
  background: rgba(251, 191, 36, 0.08);
  color: #fde68a; font-family: var(--mono); overflow-wrap: anywhere;
  user-select: all; border-radius: var(--r1);
}
details.panel summary { cursor: pointer; font-weight: 650; }

/* Login */
.login-wrap {
  min-height: 100vh; display: flex; align-items: center; justify-content: center;
  padding: var(--s5);
}
.login-card { width: min(400px, 100%); padding: var(--s6); border: 1px solid rgba(255, 255, 255, 0.12); border-radius: var(--r3); background: rgba(16, 23, 27, 0.9); backdrop-filter: blur(20px); -webkit-backdrop-filter: blur(20px); box-shadow: var(--shadow-lg), 0 0 40px rgba(59, 209, 132, 0.08); }
.login-card h1 { margin-bottom: var(--s2); text-align: center; }
.login-card input { width: 100%; margin: var(--s3) 0; padding: 11px var(--s3); border: 1px solid var(--line-strong); border-radius: var(--r1); background: var(--bg); color: var(--text); font: inherit; }
.login-card .field { margin: var(--s4) 0; }
.login-card .field input { margin: 0; }
.login-card button.primary { width: 100%; }

/* Responsive Media Queries */
@media (max-width: 1080px) {
  .dashboard-grid, .signer-grid { grid-template-columns: 1fr; }
  .resource-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .resource:nth-child(2) { border-right: 0; }
  .resource:nth-child(-n+2) { border-bottom: 1px solid var(--line); }
}

@media (max-width: 900px) {
  .sidebar { display: none; }
  .content { padding: 24px 18px 48px; }
  .mobile-nav {
    position: sticky; z-index: 20; top: 0; display: block; padding: 0 16px;
    border-bottom: 1px solid var(--line); background: rgba(10, 15, 18, 0.94);
    backdrop-filter: blur(14px); -webkit-backdrop-filter: blur(14px);
  }
  .mobile-nav summary { padding: 8px 0; }
  .mobile-brand {
    display: inline-flex; align-items: center; min-height: 36px;
    margin: 0; padding: 4px 14px 4px 38px; white-space: nowrap;
  }
  .mobile-brand::before { left: 5px; top: 6px; }
  .mobile-nav .nav-group, .mobile-nav .nav-group.utility { margin: 0 0 var(--s2); padding: 0; border: 0; }
  .mobile-nav .nav-title { padding-left: 8px; }
  .workspace-bar { margin-top: -4px; }
}

@media (max-width: 680px) {
  h1 { font-size: 22px; }
  .page-head { align-items: stretch; }
  .page-head .toolbar { width: 100%; }
  .page-head .toolbar > *, .page-head .toolbar form, .page-head .toolbar button { flex: 1; }
  .cards, .stat-grid { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .card, .stat { border-right: 1px solid var(--line); border-bottom: 1px solid var(--line); }
  .card:nth-child(2n), .stat:nth-child(2n) { border-right: 0; }
  .card:nth-last-child(-n+2), .stat:nth-last-child(-n+2) { border-bottom: 0; }
  .dashboard-table { border: 0; background: transparent; }
  .dashboard-table thead { display: none; }
  .dashboard-table tbody, .dashboard-table tr, .dashboard-table td { display: block; width: 100%; }
  .dashboard-table tr {
    margin-bottom: var(--s3); padding: var(--s3) var(--s4);
    border: 1px solid var(--line); border-radius: var(--r2); background: var(--panel);
  }
  .dashboard-table td {
    min-height: 0; height: auto; display: grid;
    grid-template-columns: 112px minmax(0, 1fr); gap: var(--s3); padding: 7px 0; border: 0;
  }
  .dashboard-table td::before {
    content: attr(data-label); color: var(--faint); font-size: 10px;
    font-weight: 700; letter-spacing: .07em; text-transform: uppercase;
  }
  .dashboard-table td:first-child { display: block; padding-bottom: 11px; border-bottom: 1px solid var(--line); }
  .dashboard-table td:first-child::before { display: none; }
  .dashboard-table td:last-child { display: block; padding-top: 11px; border-top: 1px solid var(--line); }
  .dashboard-table td:last-child::before { display: none; }
  .dashboard-table .row-actions { justify-content: flex-start; }
  .resource-strip { grid-template-columns: 1fr 1fr; }
  .surface, .panel { padding: var(--s4); }
  .filters { align-items: stretch; }
  .filters .field, .filters .field input, .filters .field select { width: 100%; }
  .filter-check { width: 100%; min-height: 44px; }
  button, .btn, .button { min-height: 44px; }
  .scroll-x { margin-right: -18px; border-right: 0; border-radius: var(--r2) 0 0 var(--r2); }
}

@media (prefers-reduced-motion: no-preference) {
  a, button, .btn, .nav-item, .field input, .field select, .field textarea, tbody tr, .card, .stat, .surface, .panel {
    transition: background-color .16s ease, border-color .16s ease, color .16s ease, box-shadow .16s ease, transform .16s ease;
  }
}

/* Node Binary & Runtime Intelligence */
/* Node Binary & Runtime Intelligence */
.runtime-helper-wrap { margin-top: -6px; margin-bottom: var(--s2); }
.binary-inference-badge { display: inline-flex; align-items: center; gap: 6px; padding: 4px 10px; margin-bottom: 8px; border-radius: var(--r1); font: 550 12px/1.4 var(--sans); background: rgba(46, 194, 116, 0.12); border: 1px solid rgba(46, 194, 116, 0.35); color: var(--jade-strong); }
.runtime-picker-row { display: flex; flex-direction: column; gap: 8px; padding: 10px 14px; background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r2); }
.runtime-picker-title { font-size: 11px; font-weight: 600; text-transform: uppercase; letter-spacing: 0.06em; color: var(--muted); }
.runtime-chips { display: flex; flex-wrap: wrap; gap: 8px; }
.runtime-chip { display: inline-flex; align-items: center; gap: 6px; padding: 6px 12px; background: var(--panel); border: 1px solid var(--line); border-radius: var(--r1); color: var(--fg); font: 500 12px/1.3 var(--mono); cursor: pointer; transition: all 0.15s ease; }
.runtime-chip:hover { background: rgba(46, 194, 116, 0.08); border-color: var(--jade); color: #fff; transform: translateY(-1px); }
.runtime-chip-active { background: rgba(46, 194, 116, 0.18); border-color: var(--jade-strong); color: var(--jade-strong); box-shadow: 0 0 10px var(--jade-glow); }
.runtime-catalogue-box { display: flex; align-items: center; justify-content: space-between; gap: 16px; padding: 10px 14px; background: rgba(46, 194, 116, 0.05); border: 1px dashed rgba(46, 194, 116, 0.3); border-radius: var(--r2); }
.runtime-catalogue-desc { display: flex; align-items: center; gap: 10px; font-size: 12px; color: var(--muted); }
.runtime-catalogue-desc strong { color: #fff; }
.runtime-catalogue-desc code { color: var(--jade-strong); background: var(--panel-2); padding: 2px 5px; border-radius: var(--r1); }

/* Node Creation Role Presets & Capability Matrix */
.role-presets-section { margin-bottom: var(--s4); padding-bottom: var(--s3); border-bottom: 1px solid var(--line); }
.section-lead-label { display: flex; align-items: flex-start; gap: 12px; margin-bottom: var(--s3); }
.section-lead-label .lead-icon { font-size: 20px; line-height: 1.2; }
.section-lead-label strong { font-size: 14px; color: var(--fg); }
.role-presets-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
.role-preset-card {
  display: flex; flex-direction: column; text-align: left; padding: 14px 16px;
  background: var(--panel); border: 1px solid var(--line); border-radius: var(--r2);
  cursor: pointer; transition: all 0.16s ease; position: relative; overflow: hidden;
}
.role-preset-card:hover {
  border-color: var(--jade); background: rgba(46, 194, 116, 0.04);
  transform: translateY(-2px); box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}
.role-preset-card.active {
  border-color: var(--jade-strong); background: rgba(46, 194, 116, 0.1);
  box-shadow: 0 0 16px var(--jade-glow);
}
.role-card-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: 8px; }
.role-card-icon { font-size: 20px; }
.role-card-badge {
  font-size: 10px; font-weight: 600; text-transform: uppercase; padding: 2px 6px;
  border-radius: 4px; background: rgba(255, 255, 255, 0.06); color: var(--muted);
}
.role-preset-card.active .role-card-badge { background: rgba(46, 194, 116, 0.25); color: var(--jade-strong); }
.role-card-title { font-weight: 600; font-size: 13px; color: var(--fg); margin-bottom: 4px; }
.role-preset-card.active .role-card-title { color: #fff; }
.role-card-desc { font-size: 11px; line-height: 1.4; color: var(--muted); }

/* Conditional RPC Service Box */
.rpc-capability-box {
  background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r2);
  padding: 16px 20px; margin: var(--s2) 0;
}
.rpc-toggle-header { display: flex; align-items: center; justify-content: space-between; gap: 16px; }
.toggle-control { display: inline-flex; align-items: center; gap: 12px; cursor: pointer; user-select: none; }
.toggle-control input[type="checkbox"] {
  width: 38px; height: 20px; appearance: none; background: var(--line); border-radius: 12px;
  position: relative; outline: none; cursor: pointer; transition: background 0.2s ease; margin: 0; flex-shrink: 0;
}
.toggle-control input[type="checkbox"]:checked { background: var(--jade); }
.toggle-control input[type="checkbox"]::after {
  content: ""; position: absolute; top: 2px; left: 2px; width: 16px; height: 16px;
  background: #fff; border-radius: 50%; transition: transform 0.2s ease;
}
.toggle-control input[type="checkbox"]:checked::after { transform: translateX(18px); }
.toggle-label-text { font-size: 13px; color: var(--fg); }
.rpc-config-body { margin-top: 14px; padding-top: 14px; border-top: 1px solid rgba(255, 255, 255, 0.05); }
.rpc-ports-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 16px; }
.rpc-disabled-banner {
  display: flex; align-items: center; gap: 12px; margin-top: 12px; padding: 10px 14px;
  background: rgba(234, 179, 8, 0.08); border: 1px solid rgba(234, 179, 8, 0.25);
  border-radius: var(--r1); font-size: 12px; color: #e2e8f0;
}
.rpc-disabled-banner .notice-icon { font-size: 18px; }
.rpc-disabled-banner strong { color: #fbbf24; }

/* Plugins Selection Grid */
.plugins-selection-section { margin-top: var(--s3); padding-top: var(--s3); border-top: 1px solid var(--line); }
.plugins-selection-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 10px; }
.plugin-selection-card {
  display: flex; align-items: flex-start; gap: 12px; padding: 10px 12px;
  background: var(--panel); border: 1px solid var(--line); border-radius: var(--r1);
  cursor: pointer; transition: all 0.15s ease;
}
.plugin-selection-card:hover { border-color: var(--jade); background: rgba(46, 194, 116, 0.05); }
.plugin-selection-card input[type="checkbox"] { margin-top: 3px; flex-shrink: 0; }
.plugin-card-content { flex: 1; min-width: 0; }
.plugin-card-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 2px; }
.plugin-name { font-size: 12px; font-weight: 600; color: var(--fg); }
.plugin-desc { font-size: 11px; color: var(--muted); line-height: 1.35; }

/* Hermes Agent & Endpoints Cards */
.hermes-agent-card { border-left: 3px solid #6366f1; background: rgba(99, 102, 241, 0.03); }
.endpoints-card { border-left: 3px solid #0ea5e9; background: rgba(14, 165, 233, 0.03); }
.hermes-provision-box { background: rgba(99, 102, 241, 0.04); border: 1px solid rgba(99, 102, 241, 0.2); border-radius: var(--r1); padding: 14px; }

/* AWS Console Global Top Bar, Tabs, and Action Ribbon */
.aws-top-bar {
  position: sticky; top: 0; z-index: 90; display: flex; align-items: center; justify-content: space-between;
  height: 44px; padding: 0 var(--s4); background: #0c1216; border-bottom: 1px solid var(--line);
}
.aws-top-bar-left, .aws-top-bar-right { display: flex; align-items: center; gap: 12px; }
.aws-console-brand { display: flex; align-items: center; gap: 8px; font-weight: 700; color: #fff; text-decoration: none; font-size: 14px; }
.aws-console-brand:hover { text-decoration: none; color: #fff; }
.aws-brand-hex { color: var(--jade); font-size: 16px; }
.aws-service-label { font-size: 12px; font-weight: 500; color: var(--muted); border-left: 1px solid var(--line); padding-left: 10px; margin-left: 4px; }
.aws-global-search {
  position: relative; display: flex; align-items: center; background: #131b20; border: 1px solid var(--line-strong);
  border-radius: var(--r1); width: clamp(260px, 32vw, 480px); height: 28px; padding: 0 8px; gap: 6px;
}
.aws-global-search:focus-within { border-color: var(--jade); }
.aws-global-search input {
  background: transparent; border: none; color: var(--text); font-size: 12px; width: 100%; outline: none;
}
.aws-search-kbd {
  font-size: 10px; color: var(--muted); background: rgba(255,255,255,0.06); padding: 1px 5px; border-radius: 3px; font-family: var(--mono);
}
.aws-nav-action { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted); text-decoration: none; }
.aws-nav-action:hover { color: #fff; text-decoration: none; }
.aws-copilot-pill {
  font-size: 11px; font-weight: 600; padding: 2px 8px; border-radius: 10px; background: rgba(99, 102, 241, 0.15);
  border: 1px solid rgba(99, 102, 241, 0.35); color: #a5b4fc;
}
.aws-region-pill, .aws-account-badge {
  display: flex; align-items: center; gap: 5px; font-size: 11px; color: var(--muted); background: rgba(255,255,255,0.04);
  padding: 3px 8px; border-radius: 4px; border: 1px solid var(--line);
}
.aws-region-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--jade); display: inline-block; }
.aws-tab-bar { display: flex; gap: 2px; border-bottom: 2px solid var(--line); overflow-x: auto; margin-bottom: 16px; }
.aws-tab-btn {
  background: transparent; border: none; border-bottom: 2px solid transparent; margin-bottom: -2px;
  padding: 8px 14px; font-size: 13px; font-weight: 500; color: var(--muted); cursor: pointer; transition: all .15s ease;
}
.aws-tab-btn:hover { color: #fff; }
.aws-tab-btn.active { color: var(--jade); border-bottom-color: var(--jade); font-weight: 600; }
.aws-summary-card { border-left: 3px solid var(--jade); }
.aws-action-bar { background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r1); }
.aws-wizard-card { margin-bottom: 14px; background: var(--panel); border: 1px solid var(--line); border-radius: var(--r2); }
.aws-wizard-summary { border-top: 2px solid var(--jade); }
.aws-log-terminal { background: #07090e; border: 1px solid rgba(255,255,255,0.06); border-radius: 4px; padding: 12px; font-family: var(--mono); max-height: 260px; overflow-y: auto; }
.aws-log-line { font-size: 12px; line-height: 1.5; color: #d1d5db; white-space: pre-wrap; word-break: break-all; }
.aws-log-num { color: var(--muted); display: inline-block; width: 42px; user-select: none; }
.aws-alarm-card { background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r1); padding: 12px 16px; display: flex; align-items: center; justify-content: space-between; }
.aws-alarm-ok { border-left: 3px solid var(--jade); }
.aws-alarm-danger { border-left: 3px solid var(--red); }
.aws-alarm-warn { border-left: 3px solid var(--amber); }
.aws-metric-gauge { height: 6px; background: rgba(255,255,255,0.08); border-radius: 3px; overflow: hidden; margin-top: 6px; }
.aws-metric-progress { height: 100%; border-radius: 3px; transition: width 0.3s ease; }
.aws-time-selector { display: flex; align-items: center; gap: 4px; background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r1); padding: 2px 4px; }
.aws-time-pill { font-size: 11px; font-weight: 500; color: var(--muted); padding: 3px 8px; border-radius: 3px; text-decoration: none; }
.aws-time-pill.active, .aws-time-pill:hover { background: rgba(255,255,255,0.08); color: #fff; text-decoration: none; }
.aws-ops-card { background: var(--panel); border: 1px solid var(--line); border-radius: var(--r1); padding: 14px; margin-bottom: 10px; }
.aws-detail-drawer { margin-top: 16px; background: var(--panel-2); border: 1px solid var(--line); border-radius: var(--r2); padding: 16px; }
.aws-drawer-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 14px; font-size: 12px; }
/* AWS Hyperscaler Industrial UI & Interactive Selection */
tr.aws-row-selected { background: rgba(59, 209, 132, 0.08) !important; border-left: 3px solid var(--jade) !important; }
tbody tr[data-node-id] { cursor: pointer; transition: background 0.12s ease; }
tbody tr[data-node-id]:hover { background: rgba(255, 255, 255, 0.04); }
.aws-pulse-dot { display: inline-block; width: 8px; height: 8px; border-radius: 50%; background: var(--jade); position: relative; margin-right: 6px; }
.aws-pulse-dot::after { content: ""; position: absolute; inset: -3px; border-radius: 50%; border: 2px solid var(--jade); opacity: 0.75; animation: aws-ping 2.2s cubic-bezier(0,0,0.2,1) infinite; }
@keyframes aws-ping { 75%, 100% { transform: scale(2.2); opacity: 0; } }
.aws-service-menu-wrap { position: relative; }
.aws-service-menu { display: none; position: absolute; top: calc(100% + 4px); left: 0; z-index: 100; min-width: 320px; background: #0c131a; border: 1px solid #1f3144; border-radius: var(--r1); box-shadow: var(--shadow-lg); padding: 10px; }
.aws-service-menu-wrap:focus-within .aws-service-menu, .aws-service-menu-wrap:hover .aws-service-menu { display: block; }
.aws-service-group { margin-bottom: 8px; padding-bottom: 6px; border-bottom: 1px solid rgba(255,255,255,0.06); }
.aws-service-group:last-child { margin-bottom: 0; padding-bottom: 0; border-bottom: none; }
.aws-service-group-title { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: .08em; color: #ff9900; margin-bottom: 4px; padding: 2px 6px; }
.aws-service-link { display: flex; align-items: center; gap: 8px; padding: 5px 8px; border-radius: 4px; color: #e2e8f0; font-size: 12px; text-decoration: none; }
.aws-service-link:hover { background: rgba(255,255,255,0.08); color: #fff; text-decoration: none; }
.aws-chart-box { background: #080d11; border: 1px solid var(--line); border-radius: var(--r1); padding: 10px 14px; margin-top: 8px; }
.aws-chart-header { display: flex; justify-content: space-between; font-size: 11px; color: var(--muted); margin-bottom: 6px; }
.aws-sparkline { width: 100%; height: 42px; display: block; overflow: visible; }
.aws-log-header { display: flex; align-items: center; justify-content: space-between; background: #0a0f14; border: 1px solid rgba(255,255,255,0.08); border-bottom: none; border-radius: 4px 4px 0 0; padding: 8px 12px; font-size: 11px; }
.aws-log-live-dot { width: 7px; height: 7px; border-radius: 50%; background: #3bd184; display: inline-block; margin-right: 5px; box-shadow: 0 0 6px #3bd184; }
.breadcrumb { display: flex; align-items: center; gap: 6px; font-size: 12px; color: var(--muted); margin-bottom: 12px; }
.breadcrumb a { color: var(--muted); text-decoration: none; transition: color .12s ease; }
.breadcrumb a:hover { color: #539fe5; text-decoration: underline; }
.breadcrumb .crumb-sep { color: rgba(255,255,255,0.25); font-size: 10px; margin: 0 2px; }
table.dashboard-table thead th { position: sticky; top: 0; background: #0d1520; z-index: 2; }
:focus-visible { outline: 2px solid #539fe5; outline-offset: 1px; }
"#;
