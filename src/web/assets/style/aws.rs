//! The AWS-console-style top bar, tabs, action ribbon and hyperscaler selection surfaces.
//!
//! One slice of the console stylesheet. Slices are concatenated in source
//! order by `super::css`; see `style.rs` for why the sheet is not one file.

pub(super) const RULES: &str = r#"/* AWS Console Global Top Bar, Tabs, and Action Ribbon */
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
/* The pending marker sits next to the button of a form that takes a while.
   It is hidden until the bundled script sees the form being submitted, so
   the page is unchanged for an operator without scripting. */
.pending { display: none; margin-left: 10px; font-size: 12px; color: var(--amber); }
form[data-pending-state="running"] .pending { display: inline-flex; align-items: center; gap: 6px; }
form[data-pending-state="running"] .pending::before {
  content: "";
  width: 10px;
  height: 10px;
  border-radius: 50%;
  border: 2px solid currentColor;
  border-right-color: transparent;
  animation: pending-spin 0.8s linear infinite;
}
@keyframes pending-spin { to { transform: rotate(360deg); } }
@media (prefers-reduced-motion: reduce) {
  form[data-pending-state="running"] .pending::before { animation: none; }
}
"#;
