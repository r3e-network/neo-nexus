//! NeoNexus operations-console visual system.

pub const CSS: &str = r#"
:root {
  color-scheme: dark;
  --bg: #090d0f; --panel: #0e1417; --panel-2: #121a1e; --panel-3: #172126;
  --line: #223038; --line-strong: #30434d;
  --text: #e8f0ed; --muted: #93a39d; --faint: #65756f;
  --jade: #3bd184; --jade-strong: #55e398; --jade-ink: #06140c;
  --cyan: #4bb7d3; --amber: #dca94a; --red: #ef6672; --idle: #788780;
  --s1: 4px; --s2: 8px; --s3: 12px; --s4: 16px; --s5: 24px; --s6: 32px;
  --r1: 5px; --r2: 8px;
  --sans: "Segoe UI Variable", "Segoe UI", Inter, system-ui, -apple-system, sans-serif;
  --mono: ui-monospace, "Cascadia Mono", "SF Mono", Consolas, "Liberation Mono", monospace;
}
* { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body { margin: 0; background: var(--bg); color: var(--text); font: 14px/1.55 var(--sans); }
.skip-link { position: fixed; z-index: 100; left: var(--s3); top: var(--s3); padding: 8px 12px;
  transform: translateY(-180%); border: 1px solid var(--jade); border-radius: var(--r1);
  background: var(--panel); color: var(--text); }
.skip-link:focus { transform: translateY(0); }
h1 { margin: 0 0 var(--s4); font-size: clamp(22px, 2vw, 28px); line-height: 1.2;
  font-weight: 650; letter-spacing: -.025em; }
.page-head h1 { margin-bottom: 0; }
h2 { margin: var(--s6) 0 var(--s3); color: var(--text); font-size: 14px;
  font-weight: 650; letter-spacing: .01em; }
h3 { margin: 0 0 var(--s3); font-size: 14px; font-weight: 650; }
p { margin: 0 0 var(--s4); }
a { color: var(--text); text-decoration-color: var(--line-strong); text-underline-offset: 3px; }
a:hover { color: var(--jade); text-decoration-color: currentColor; }
:focus-visible { outline: 2px solid var(--jade); outline-offset: 3px; border-radius: var(--r1); }
code, .mono, .num, .path { font-family: var(--mono); font-size: 12.5px; }
.num, td.num, .stat-value { font-variant-numeric: tabular-nums; }

/* Shell */
.shell { display: flex; min-height: 100vh; }
.sidebar { position: sticky; top: 0; width: 248px; height: 100vh; flex: 0 0 248px;
  overflow-y: auto; background: #0b1114; border-right: 1px solid var(--line);
  padding: 20px 14px 14px; display: flex; flex-direction: column; gap: 2px; }
.brand { position: relative; padding: 2px 12px 24px 40px; color: #f3f8f5;
  font-weight: 700; font-size: 15px; letter-spacing: .02em; text-decoration: none; }
.brand::before { content: "N"; position: absolute; left: 8px; top: -2px; width: 24px;
  height: 24px; display: grid; place-items: center; border: 1px solid var(--jade);
  border-radius: 6px; color: var(--jade); font: 700 12px/1 var(--mono); }
.nav-group { margin-bottom: var(--s3); }
.sidebar-nav { min-height: 0; display: flex; flex: 1; flex-direction: column; }
.nav-group.utility { margin-top: auto; padding-top: var(--s3); border-top: 1px solid var(--line); }
.nav-title { padding: 10px 10px 5px; color: var(--faint); font-size: 10px; font-weight: 700;
  text-transform: uppercase; letter-spacing: .12em; }
.nav-item { display: flex; align-items: center; gap: 10px; width: 100%; min-height: 36px;
  padding: 7px 10px; border: 0; border-left: 2px solid transparent;
  border-radius: var(--r1); background: transparent; color: var(--muted); text-align: left;
  text-decoration: none; font: inherit; font-size: 13px; cursor: pointer; }
.nav-item:hover { background: var(--panel-2); color: var(--text); }
.nav-item.current { border-left-color: var(--jade); background: #111d19; color: #f0f8f3; }
.nav-icon { width: 16px; height: 16px; flex: 0 0 16px; color: var(--faint); }
.nav-item.current .nav-icon, .nav-item:hover .nav-icon { color: var(--jade); }
.sidebar-utilities { margin-top: var(--s2); padding-top: var(--s2); border-top: 1px solid var(--line); }
.sidebar-utilities form, .mobile-utilities form { margin: 0; }
.logout { margin-top: 2px; color: var(--faint); }
.content { width: 100%; max-width: 1600px; min-width: 0;
  padding: 34px clamp(24px, 4vw, 64px) 56px; }
.mobile-nav { display: none; }
.mobile-nav summary { list-style: none; min-height: 44px; display: flex; align-items: center;
  justify-content: space-between; cursor: pointer; }
.mobile-nav summary::-webkit-details-marker { display: none; }
.mobile-nav-menu { border-top: 1px solid var(--line); padding: var(--s3); }
.mobile-nav details[open] .mobile-nav-menu { max-height: calc(100vh - 61px); overflow-y: auto; }
.mobile-utilities { margin-top: var(--s2); padding-top: var(--s2); border-top: 1px solid var(--line); }
.menu-label { color: var(--muted); font-size: 12px; font-weight: 650; letter-spacing: .06em;
  text-transform: uppercase; }
.workspace-bar { min-height: 34px; display: flex; align-items: center; justify-content: space-between;
  gap: var(--s3); margin: -10px 0 var(--s5); padding-bottom: var(--s3); border-bottom: 1px solid var(--line);
  color: var(--faint); font-size: 11.5px; }
.workspace-bar span { display: inline-flex; align-items: center; gap: 8px; }
.workspace-bar i { width: 6px; height: 6px; border-radius: 50%; background: var(--jade); }
.workspace-bar a { color: var(--muted); }

/* Page furniture */
.breadcrumb { margin-bottom: var(--s2); color: var(--muted); font-size: 12.5px; }
.breadcrumb a { color: var(--muted); }
.breadcrumb .sep { margin: 0 var(--s1); color: var(--faint); }
.page-head { display: flex; align-items: flex-start; justify-content: space-between; gap: var(--s4);
  flex-wrap: wrap; margin-bottom: var(--s5); padding-bottom: var(--s5); border-bottom: 1px solid var(--line); }
.page-head .sub { max-width: 760px; margin-top: 5px; color: var(--muted); font-size: 13px; }
.toolbar, .actions, .row-actions { display: flex; align-items: center; gap: var(--s2); flex-wrap: wrap; }
.row-actions { justify-content: flex-end; white-space: nowrap; }
.node-controls { margin-bottom: 20px; }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: var(--s3);
  margin-bottom: var(--s4); }
.section-head h2 { margin: 0; }
.surface, .panel { background: var(--panel); border: 1px solid var(--line);
  border-radius: var(--r2); padding: var(--s5); }
.surface > :first-child, .panel > :first-child { margin-top: 0; }
.surface > :last-child, .panel > :last-child { margin-bottom: 0; }

/* Statistics */
.cards, .stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  margin: 0 0 var(--s5); border-top: 1px solid var(--line); border-bottom: 1px solid var(--line); }
.card, .stat { min-width: 0; padding: 17px var(--s4); border-right: 1px solid var(--line);
  background: transparent; }
.card:last-child, .stat:last-child { border-right: 0; }
.card .num, .stat-value { color: #f2f8f5; font-size: 24px; font-weight: 650; line-height: 1.15; }
.card .lbl, .stat-label { margin-top: 5px; color: var(--muted); font-size: 10.5px;
  font-weight: 650; letter-spacing: .08em; text-transform: uppercase; }
.stat-detail { margin-top: 4px; color: var(--faint); font-size: 11.5px; }
.stat.positive .stat-value { color: var(--jade); }
.stat.info .stat-value { color: var(--cyan); }
.stat.warning .stat-value { color: var(--amber); }
.stat.danger .stat-value { color: var(--red); }

/* Tables */
.scroll-x { overflow-x: auto; border: 1px solid var(--line); border-radius: var(--r2); }
.scroll-x table { min-width: 640px; border: 0; }
table { width: 100%; border-collapse: collapse; border: 1px solid var(--line); background: var(--panel); }
th, td { padding: 11px var(--s3); border-bottom: 1px solid var(--line); text-align: left; vertical-align: middle; }
th { color: var(--faint); background: #0c1215; font-size: 10px; font-weight: 700;
  letter-spacing: .08em; text-transform: uppercase; white-space: nowrap; }
tbody tr:last-child td { border-bottom: 0; }
tbody tr:hover td { background: #10191c; }
td.num, th.num { text-align: right; }
td.path, td.mono { color: var(--muted); word-break: break-all; }
.node-name { display: block; font-weight: 650; text-decoration: none; }
.node-meta { display: block; margin-top: 2px; color: var(--faint); font-size: 11.5px; }
.dashboard-table td { height: 58px; }

/* Density — content-only modifiers; chrome (sidebar, header) stays invariant.
   Comfortable keeps the current baseline; compact tightens row padding and the
   XS/SM spacing tokens so an inventory row settles near a 40px slot. */
.density-comfortable { --xs: var(--s2); --sm: var(--s3); --row-pad-y: 11px; }
.density-compact { --xs: var(--s1); --sm: var(--s2); --row-pad-y: 8px; }
.density-compact td, .density-compact th { padding-top: var(--row-pad-y); padding-bottom: var(--row-pad-y); }
.density-compact .card, .density-compact .stat { padding-top: var(--sm); padding-bottom: var(--sm); }
.node-line { display: flex; align-items: center; gap: var(--sm, var(--s3)); min-height: 22px; }
.node-line .node-name { font-weight: 650; text-decoration: none; }
.node-line .node-port { margin-left: auto; color: var(--muted); }
.status-dot { width: 8px; height: 8px; flex: 0 0 8px; border-radius: 50%; background: var(--idle); }
.status-dot.running { background: var(--jade); }
.status-dot.starting { background: var(--amber); }
.status-dot.error { background: var(--red); }

/* Status and intent */
.badge { display: inline-flex; align-items: center; gap: 6px; padding: 2px 8px;
  border: 1px solid var(--line-strong); border-radius: 999px; color: var(--muted);
  font-size: 11px; font-weight: 650; letter-spacing: .02em; }
.badge::before { content: ""; width: 5px; height: 5px; border-radius: 50%; background: currentColor; }
.badge.running { color: var(--jade); border-color: #245d42; background: #0d1d16; }
.badge.starting { color: var(--amber); border-color: #5b4928; background: #1d190f; }
.badge.error { color: var(--red); border-color: #63323a; background: #211216; }
.badge.stopped, .badge.unknown { color: var(--idle); }
.badge.event-info { color: var(--cyan); border-color: #285461; background: #0d181c; }
.badge.event-warning { color: var(--amber); border-color: #5b4928; background: #1d190f; }
.badge.event-critical { color: var(--red); border-color: #63323a; background: #211216; }
.tone-info { color: var(--cyan); }
.tone-warning { color: var(--amber); }
.tone-danger { color: var(--red); }
button, .btn, .button { min-height: 34px; display: inline-flex; align-items: center;
  justify-content: center; gap: 6px; padding: 6px 13px; border: 1px solid var(--line-strong);
  border-radius: var(--r1); background: var(--panel-2); color: var(--text); font: inherit;
  font-size: 13px; line-height: 1.35; text-decoration: none; cursor: pointer; }
button:hover, .btn:hover, .button:hover { border-color: var(--muted); background: var(--panel-3); color: var(--text); }
button:disabled, .btn.disabled { opacity: .42; cursor: not-allowed; }
button.primary, .btn.primary { border-color: var(--jade); background: var(--jade);
  color: var(--jade-ink); font-weight: 700; }
button.primary:hover, .btn.primary:hover { border-color: var(--jade-strong); background: var(--jade-strong); }
button.danger, .btn.danger { border-color: #63323a; background: transparent; color: var(--red); }
button.danger:hover, .btn.danger:hover { border-color: var(--red); background: #211216; }
.btn.small, button.small { min-height: 28px; padding: 3px 9px; font-size: 12px; }

/* Forms */
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(232px, 1fr));
  gap: var(--s3) var(--s4); align-items: start; }
.grid .span-all { grid-column: 1 / -1; }
fieldset { border: 0; padding: 0; margin: 0; }
.field { display: flex; flex-direction: column; gap: 5px; margin-bottom: var(--s3);
  color: var(--muted); font-size: 12px; }
.field > span, .field label { color: var(--muted); font-weight: 650; letter-spacing: .01em; }
.field input, .field select, .field textarea { width: 100%; min-width: 0; padding: 8px 10px;
  border: 1px solid var(--line-strong); border-radius: var(--r1); background: #080c0e;
  color: var(--text); font: inherit; }
.field input.mono, .field input[name=binary_path], .field input[name=runtime_version] {
  font-family: var(--mono); font-size: 12.5px; }
.field input:hover, .field select:hover, .field textarea:hover { border-color: var(--muted); }
.field input:focus, .field select:focus, .field textarea:focus { border-color: var(--jade); }
.field input:focus-visible, .field select:focus-visible, .field textarea:focus-visible {
  outline: 2px solid var(--jade); outline-offset: 2px; }
.field .help { color: var(--faint); font-size: 11.5px; font-weight: 400; }
.field .error { color: var(--red); font-size: 12px; font-weight: 500; }
.field.invalid input, .field.invalid select { border-color: var(--red); }
.filters { display: flex; align-items: flex-end; gap: var(--s2); flex-wrap: wrap; margin-bottom: var(--s4); }
.filters .field { margin-bottom: 0; }
.filters .field input, .filters .field select { width: auto; min-width: 150px; }
.filter-check { min-height: 36px; display: inline-flex; align-items: center; gap: 8px;
  padding: 7px 9px; border: 1px solid var(--line-strong); border-radius: var(--r1);
  color: var(--muted); cursor: pointer; }
.filter-check:hover { border-color: var(--muted); color: var(--text); }
.filter-check input { width: 16px; height: 16px; margin: 0; accent-color: var(--jade); }
.form-actions { display: flex; align-items: center; gap: var(--s2); flex-wrap: wrap;
  margin-top: var(--s2); padding-top: var(--s4); border-top: 1px solid var(--line); }
.form-actions .spacer { flex: 1; }

/* Messages */
.flash, .notice { margin-bottom: var(--s4); padding: 11px var(--s4); border: 1px solid var(--line);
  border-left: 3px solid var(--cyan); border-radius: var(--r1); background: var(--panel);
  color: var(--muted); font-size: 13px; }
.notice.warn { border-left-color: var(--amber); }
.notice.danger { border-left-color: var(--red); }
.empty { padding: 48px var(--s5); border: 1px dashed var(--line-strong); border-radius: var(--r2);
  background: var(--panel); text-align: center; color: var(--muted); }
.empty h2 { margin: 0 0 var(--s2); color: var(--text); font-size: 15px; }
.empty .actions { justify-content: center; margin-top: var(--s4); }
.muted { color: var(--muted); }
.err { color: var(--red); font-size: 13px; }
time { display: block; white-space: nowrap; font-variant-numeric: tabular-nums; }
time .elapsed { display: block; color: var(--faint); font: 11px/1.4 var(--sans); }
pre { overflow-x: auto; padding: var(--s4); border: 1px solid var(--line);
  border-radius: var(--r2); background: #080c0e; font: 12.5px/1.6 var(--mono); }
.facts { max-width: 720px; }
.facts th { width: 34%; background: transparent; }
.facts td { font-family: var(--mono); font-size: 12.5px; word-break: break-all; }

/* Fleet overview */
.dashboard-grid { display: grid; grid-template-columns: minmax(0, 1fr) minmax(300px, .72fr);
  gap: var(--s4); margin-top: var(--s5); }
.readiness-score { display: flex; align-items: flex-end; justify-content: space-between;
  gap: var(--s4); margin-bottom: var(--s3); }
.readiness-score strong { font: 650 30px/1 var(--mono); color: var(--jade); }
.readiness-score span { color: var(--muted); font-size: 12px; }
progress { width: 100%; height: 6px; border: 0; border-radius: 0; background: var(--line); }
progress::-webkit-progress-bar { background: var(--line); }
progress::-webkit-progress-value { background: var(--jade); }
progress::-moz-progress-bar { background: var(--jade); }
.issue-list, .activity-list { list-style: none; padding: 0; margin: var(--s4) 0 0; }
.issue-list li, .activity-list li { display: grid; grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--s3); padding: 10px 0; border-top: 1px solid var(--line); }
.issue-list a, .activity-list strong { font-weight: 600; text-decoration: none; }
.activity-list p { margin: 2px 0 0; color: var(--muted); font-size: 12px; }
.activity-list time { color: var(--faint); font-size: 11px; }
.resource-strip { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr));
  margin-top: var(--s4); border: 1px solid var(--line); border-radius: var(--r2); background: var(--panel); }
.resource { padding: 13px var(--s4); border-right: 1px solid var(--line); }
.resource:last-child { border-right: 0; }
.resource strong { display: block; font: 650 14px/1.3 var(--mono); }
.resource span { display: block; margin-top: 3px; color: var(--faint); font-size: 10px;
  letter-spacing: .07em; text-transform: uppercase; }

/* Signer */
.subnav { display: flex; gap: 2px; overflow-x: auto; margin: -8px 0 var(--s5);
  border-bottom: 1px solid var(--line); }
.subnav a { min-height: 38px; display: inline-flex; align-items: center; padding: 0 13px;
  border-bottom: 2px solid transparent; color: var(--muted); text-decoration: none; white-space: nowrap; }
.subnav a:hover { color: var(--text); }
.subnav a.current, .subnav a[aria-current=page] { border-bottom-color: var(--jade); color: var(--jade); }
.custody-band { display: grid; grid-template-columns: auto 1fr; gap: var(--s3); align-items: start;
  margin-bottom: var(--s5); padding: var(--s4); border: 1px solid var(--line);
  border-left: 3px solid var(--jade); border-radius: var(--r1); background: var(--panel); }
.custody-band.warn { border-left-color: var(--amber); }
.custody-band.danger { border-left-color: var(--red); }
.custody-mark { width: 26px; height: 26px; display: grid; place-items: center;
  border: 1px solid currentColor; border-radius: 50%; color: var(--jade); font: 700 12px/1 var(--mono); }
.custody-band.warn .custody-mark { color: var(--amber); }
.custody-band.danger .custody-mark { color: var(--red); }
.custody-band strong { display: block; margin-bottom: 2px; }
.custody-band p { margin: 0; color: var(--muted); font-size: 12.5px; }
.signer-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: var(--s4); }
.summary-list { list-style: none; padding: 0; margin: 0; }
.summary-list li { display: flex; justify-content: space-between; gap: var(--s3);
  padding: 10px 0; border-top: 1px solid var(--line); }
.summary-list li:first-child { border-top: 0; padding-top: 0; }
.summary-list span { color: var(--muted); }
.secret-value { display: block; margin-top: var(--s2); padding: 10px; border: 1px solid #5b4928;
  background: #17140d; color: var(--amber); font-family: var(--mono); overflow-wrap: anywhere;
  user-select: all; }
details.panel summary { cursor: pointer; font-weight: 650; }

/* Login */
.login-wrap { min-height: 100vh; display: flex; align-items: center; justify-content: center; padding: var(--s5); }
.login-card { width: min(380px, 100%); padding: var(--s6); border: 1px solid var(--line);
  border-radius: var(--r2); background: var(--panel); }
.login-card h1 { margin-bottom: var(--s2); text-align: center; }
.login-card input { width: 100%; margin: var(--s3) 0; padding: 10px var(--s3);
  border: 1px solid var(--line-strong); border-radius: var(--r1); background: var(--bg);
  color: var(--text); font: inherit; }
.login-card .field { margin: var(--s4) 0; }
.login-card .field input { margin: 0; }
.login-card button.primary { width: 100%; }

@media (max-width: 1080px) {
  .dashboard-grid, .signer-grid { grid-template-columns: 1fr; }
  .resource-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
  .resource:nth-child(2) { border-right: 0; }
  .resource:nth-child(-n+2) { border-bottom: 1px solid var(--line); }
}
@media (max-width: 900px) {
  .sidebar { display: none; }
  .content { padding: 24px 18px 44px; }
  .mobile-nav { position: sticky; z-index: 20; top: 0; display: block; padding: 0 16px;
    border-bottom: 1px solid var(--line); background: #0b1114; }
  .mobile-nav summary { padding: 8px 0; }
  .mobile-brand { display: inline-flex; align-items: center; min-height: 36px;
    margin: 0; padding: 4px 14px 4px 36px; white-space: nowrap; }
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
  .dashboard-table tr { margin-bottom: var(--s3); padding: var(--s3) var(--s4);
    border: 1px solid var(--line); border-radius: var(--r2); background: var(--panel); }
  .dashboard-table td { min-height: 0; height: auto; display: grid;
    grid-template-columns: 112px minmax(0, 1fr); gap: var(--s3); padding: 7px 0; border: 0; }
  .dashboard-table td::before { content: attr(data-label); color: var(--faint); font-size: 10px;
    font-weight: 700; letter-spacing: .07em; text-transform: uppercase; }
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
  a, button, .btn, .nav-item, .field input, .field select, .field textarea, tbody td {
    transition: background-color .12s ease, border-color .12s ease, color .12s ease;
  }
}
"#;
