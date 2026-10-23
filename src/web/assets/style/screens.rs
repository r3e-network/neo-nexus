//! The per-page compositions: fleet overview, signer, login, the responsive rules, node runtime, role presets, plugin grid and the Hermes agent cards.
//!
//! One slice of the console stylesheet. Slices are concatenated in source
//! order by `super::css`; see `style.rs` for why the sheet is not one file.

pub(super) const RULES: &str = r#"/* Fleet Overview Dashboard */
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

"#;
