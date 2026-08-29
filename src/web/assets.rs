//! Embedded stylesheet and script. The source-purity gate keeps `.css`/`.js`
//! files out of the repository, so the browser assets live here as string
//! constants and ship inside the single binary. The script only polls the
//! fleet API and swaps status badges — every control works without JavaScript.

pub const CSS: &str = r#"
:root {
  /* Surfaces */
  --bg: #14161a; --panel: #1c1f26; --panel-2: #232731; --panel-3: #2a2f3a;
  --line: #2e3340; --line-soft: #262b36;
  /* Type */
  --text: #e8eaf0; --muted: #9aa1b2; --faint: #6f7789;
  /* Intent. --accent is brand and primary action; --danger is a different hue,
     not a different shade of the same one, so Save and Delete can never be
     confused at a glance. */
  --accent: #e0604d; --accent-ink: #fff;
  --ok: #4cc38a; --warn: #d9a13b; --bad: #e0564d; --idle: #6b7280;
  --danger: #ff5470;
  /* Rhythm */
  --s1: 4px; --s2: 8px; --s3: 12px; --s4: 16px; --s5: 24px; --s6: 32px;
  --r1: 6px; --r2: 10px; --r3: 14px;
  --mono: ui-monospace, "Cascadia Mono", "SF Mono", Consolas, "Liberation Mono", monospace;
}
* { box-sizing: border-box; }
html { -webkit-text-size-adjust: 100%; }
body { margin: 0; background: var(--bg); color: var(--text);
  font: 14px/1.55 "Segoe UI", system-ui, -apple-system, sans-serif; }
h1 { font-size: 21px; line-height: 1.25; font-weight: 650; letter-spacing: -.2px;
  /* Most pages still emit a bare <h1>, so the heading carries its own space.
     Inside .page-head the block owns the spacing instead. */
  margin: 0 0 var(--s4); }
.page-head h1 { margin-bottom: 0; }
h2 { font-size: 12px; font-weight: 600; margin: var(--s5) 0 var(--s2);
  color: var(--muted); text-transform: uppercase; letter-spacing: .8px; }
a { color: var(--text); text-decoration-thickness: 1px; text-underline-offset: 2px; }
a:hover { color: var(--accent); }
:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; border-radius: 4px; }
code, .mono, .num, .path { font-family: var(--mono); font-size: 12.5px; }
.num, td.num, .stat-value { font-variant-numeric: tabular-nums; }

/* ---------- shell ---------- */
.shell { display: flex; min-height: 100vh; }
.sidebar { width: 208px; flex: 0 0 208px; background: var(--panel);
  border-right: 1px solid var(--line); padding: var(--s4) var(--s3);
  display: flex; flex-direction: column; gap: 2px; }
.brand { font-weight: 700; font-size: 15px; padding: var(--s1) 10px var(--s4);
  letter-spacing: .3px; }
.nav-group { margin-bottom: var(--s3); }
.nav-title { color: var(--faint); font-size: 10px; font-weight: 600;
  text-transform: uppercase; letter-spacing: 1px; padding: var(--s3) 10px var(--s1); }
.nav-item { display: block; padding: 7px 10px; border-radius: var(--r1);
  color: var(--muted); text-decoration: none; border: 0; background: none;
  width: 100%; text-align: left; font: inherit; font-size: 13.5px; cursor: pointer; }
.nav-item:hover { background: var(--panel-2); color: var(--text); }
.nav-item.current { background: var(--panel-2); color: var(--text);
  box-shadow: inset 2px 0 0 var(--accent); }
.logout { margin-top: auto; color: var(--faint); }
.content { flex: 1; padding: var(--s5) var(--s6); min-width: 0; max-width: 1400px; }

/* ---------- page furniture ---------- */
.breadcrumb { color: var(--muted); font-size: 12.5px; margin-bottom: var(--s2); }
.breadcrumb a { color: var(--muted); }
.breadcrumb .sep { color: var(--faint); margin: 0 var(--s1); }
.page-head { display: flex; align-items: flex-start; justify-content: space-between;
  gap: var(--s4); flex-wrap: wrap; margin-bottom: var(--s5); }
.page-head .sub { color: var(--muted); font-size: 13px; margin-top: 2px; }
.toolbar { display: flex; gap: var(--s2); align-items: center; flex-wrap: wrap; }

/* ---------- stats ---------- */
.cards { display: flex; gap: var(--s3); flex-wrap: wrap; margin-bottom: var(--s4); }
.card { background: var(--panel); border: 1px solid var(--line);
  border-radius: var(--r2); padding: var(--s3) var(--s4); min-width: 124px; }
.card .num { font-size: 22px; font-weight: 700; line-height: 1.2; }
.card .lbl { color: var(--muted); font-size: 11px; text-transform: uppercase;
  letter-spacing: .6px; margin-top: 2px; }

/* ---------- tables ---------- */
table { border-collapse: collapse; width: 100%; background: var(--panel);
  border: 1px solid var(--line); border-radius: var(--r2); overflow: hidden; }
th, td { text-align: left; padding: 9px var(--s3); border-bottom: 1px solid var(--line-soft);
  vertical-align: middle; }
th { color: var(--muted); font-size: 11px; font-weight: 600; text-transform: uppercase;
  letter-spacing: .6px; background: var(--panel-2); white-space: nowrap; }
tbody tr:last-child td, tr:last-child td { border-bottom: 0; }
tbody tr:hover td { background: rgba(255,255,255,.018); }
td.num, th.num { text-align: right; font-variant-numeric: tabular-nums; }
td.path, td.mono { font-family: var(--mono); font-size: 12.5px; color: var(--muted);
  word-break: break-all; }
.row-actions { display: flex; gap: 6px; justify-content: flex-end; white-space: nowrap; }

/* ---------- badges ---------- */
.badge { display: inline-block; padding: 2px 9px; border-radius: 999px;
  font-size: 11.5px; font-weight: 600; letter-spacing: .2px;
  border: 1px solid transparent; }
.badge.running { background: rgba(76,195,138,.14); color: var(--ok);
  border-color: rgba(76,195,138,.3); }
.badge.starting { background: rgba(217,161,59,.14); color: var(--warn);
  border-color: rgba(217,161,59,.3); }
.badge.error { background: rgba(224,86,77,.14); color: var(--bad);
  border-color: rgba(224,86,77,.32); }
.badge.stopped { background: rgba(107,114,128,.14); color: var(--idle);
  border-color: rgba(107,114,128,.3); }
.badge.unknown { background: rgba(154,161,178,.12); color: var(--muted);
  border-color: rgba(154,161,178,.26); }

/* ---------- buttons ---------- */
button, .btn { background: var(--panel-2); color: var(--text);
  border: 1px solid var(--line); border-radius: var(--r1); padding: 6px 13px;
  font: inherit; font-size: 13px; cursor: pointer; text-decoration: none;
  display: inline-flex; align-items: center; gap: 6px; line-height: 1.4; }
button:hover, .btn:hover { border-color: var(--faint); background: var(--panel-3); }
button:disabled, .btn.disabled { opacity: .45; cursor: not-allowed; }
button.primary, .btn.primary { background: var(--accent); border-color: var(--accent);
  color: var(--accent-ink); font-weight: 600; }
button.primary:hover, .btn.primary:hover { background: #e86f5d; border-color: #e86f5d; }
/* Destructive is outlined, never filled: a filled red button at the same size as
   Save invites the wrong click, and Delete is not reversible. */
button.danger, .btn.danger { background: transparent; border-color: rgba(255,84,112,.5);
  color: var(--danger); }
button.danger:hover, .btn.danger:hover { background: rgba(255,84,112,.12);
  border-color: var(--danger); }
.btn.small, button.small { padding: 3px 9px; font-size: 12px; }
.actions { display: flex; gap: var(--s2); align-items: center; flex-wrap: wrap; }

/* ---------- forms ---------- */
.grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(232px, 1fr));
  gap: var(--s3) var(--s4); align-items: start; }
.grid .span-all { grid-column: 1 / -1; }
fieldset { border: 0; padding: 0; margin: 0; }
.field { display: flex; flex-direction: column; gap: 5px; font-size: 12px;
  color: var(--muted); margin-bottom: var(--s3); }
.field > span, .field label { font-weight: 600; letter-spacing: .2px; color: var(--muted); }
.field input, .field select, .field textarea { background: var(--bg); color: var(--text);
  border: 1px solid var(--line); border-radius: var(--r1); padding: 7px 10px;
  font: inherit; width: 100%; min-width: 0; }
.field input.mono, .field input[name=binary_path], .field input[name=runtime_version] {
  font-family: var(--mono); font-size: 12.5px; }
.field input:hover, .field select:hover { border-color: var(--faint); }
.field input:focus, .field select:focus { border-color: var(--accent); outline: none; }
.field .help { color: var(--faint); font-size: 11.5px; font-weight: 400; }
.field .error { color: var(--danger); font-size: 12px; font-weight: 500; }
.field.invalid input, .field.invalid select { border-color: var(--danger); }
.filters { display: flex; gap: var(--s2); flex-wrap: wrap; align-items: flex-end;
  margin-bottom: var(--s4); }
.filters .field { margin-bottom: 0; }
.filters .field input, .filters .field select { min-width: 150px; width: auto; }
.form-actions { display: flex; gap: var(--s2); align-items: center;
  margin-top: var(--s2); padding-top: var(--s4); border-top: 1px solid var(--line-soft);
  flex-wrap: wrap; }
.form-actions .spacer { flex: 1; }
.panel { background: var(--panel); border: 1px solid var(--line);
  border-radius: var(--r2); padding: var(--s4) var(--s5) var(--s5); }

/* ---------- facts ---------- */
.facts { max-width: 620px; }
.facts th { width: 34%; background: none; }
.facts td { font-family: var(--mono); font-size: 12.5px; word-break: break-all; }

/* ---------- messages ---------- */
.flash { background: var(--panel-2); border: 1px solid var(--line);
  border-left: 3px solid var(--accent); border-radius: var(--r1);
  padding: 10px var(--s4); margin-bottom: var(--s4); }
.notice { border: 1px solid var(--line); border-left: 3px solid var(--faint);
  border-radius: var(--r1); padding: 10px var(--s4); margin-bottom: var(--s3);
  background: var(--panel); color: var(--muted); font-size: 13px; }
.notice.warn { border-left-color: var(--warn); }
.notice.danger { border-left-color: var(--danger); }
.empty { background: var(--panel); border: 1px dashed var(--line);
  border-radius: var(--r2); padding: var(--s6); text-align: center; color: var(--muted); }
.empty h2 { margin: 0 0 var(--s2); color: var(--text); text-transform: none;
  letter-spacing: 0; font-size: 15px; }
.empty .actions { justify-content: center; margin-top: var(--s4); }
.muted { color: var(--muted); }
.scroll-x { overflow-x: auto; }
.scroll-x table { min-width: 640px; }
time { display: block; font-variant-numeric: tabular-nums; white-space: nowrap; }
time .elapsed { display: block; color: var(--faint); font-size: 11px;
  font-family: inherit; }
.err { color: var(--danger); font-size: 13px; }
pre { background: var(--panel); border: 1px solid var(--line);
  border-radius: var(--r2); padding: var(--s3) var(--s4); overflow-x: auto;
  font-family: var(--mono); font-size: 12.5px; line-height: 1.6; }

/* ---------- login ---------- */
.login-wrap { display: flex; align-items: center; justify-content: center;
  min-height: 100vh; padding: var(--s5); }
.login-card { background: var(--panel); border: 1px solid var(--line);
  border-radius: var(--r3); padding: var(--s6); width: 340px; }
.login-card h1 { text-align: center; margin-bottom: var(--s2); }
.login-card input { width: 100%; margin: var(--s3) 0; padding: 9px var(--s3);
  background: var(--bg); color: var(--text); border: 1px solid var(--line);
  border-radius: var(--r1); font: inherit; }
.login-card button.primary { width: 100%; justify-content: center; }

/* ---------- narrow screens ---------- */
@media (max-width: 860px) {
  .shell { flex-direction: column; }
  .sidebar { width: auto; flex: 0 0 auto; flex-direction: row; align-items: center;
    overflow-x: auto; border-right: 0; border-bottom: 1px solid var(--line);
    padding: var(--s2) var(--s3); gap: var(--s1); }
  .brand { padding: 0 var(--s3) 0 var(--s1); white-space: nowrap; }
  .nav-group { margin: 0; display: flex; gap: var(--s1); }
  .nav-group:empty { display: none; }
  .nav-title { display: none; }
  .nav-item { width: auto; white-space: nowrap; }
  .logout { margin-top: 0; }
  .content { padding: var(--s4) var(--s3); }
}
@media (prefers-reduced-motion: no-preference) {
  a, button, .btn, .nav-item, .field input, .field select, tbody tr td {
    transition: background-color .12s ease, border-color .12s ease, color .12s ease; }
}
"#;

pub const SCRIPT: &str = r#"
(function () {
  /* Progressive enhancement only: every filter and control below still works
     with scripting disabled, this just saves one click. */
  document.addEventListener("change", function (event) {
    var el = event.target;
    if (el.matches && el.matches("select[data-autosubmit]")) {
      /* form.submit() sends no submit-button value, so raise the same flag the
         explicit buttons do. The flag is a field of its own: two values under
         one name would be ambiguous. */
      var flag = el.form.querySelector(
        "input[name=" + el.getAttribute("data-autosubmit") + "]");
      if (flag) { flag.value = "1"; }
      el.form.submit();
    }
  });
  function refresh() {
    fetch("/api/fleet").then(function (r) {
      if (r.status === 401) { location.href = "/login"; return null; }
      return r.ok ? r.json() : null;
    }).then(function (data) {
      if (!data) return;
      data.nodes.forEach(function (n) {
        document.querySelectorAll('[data-node-id="' + n.id + '"] [data-node-status]')
          .forEach(function (el) {
            el.textContent = n.status;
            el.className = "badge " + n.status.toLowerCase();
          });
        var rpc = document.querySelector(
          '[data-node-id="' + n.id + '"] [data-node-rpc]');
        if (rpc) rpc.textContent = n.rpc_health;
      });
    }).catch(function () {});
  }
  if (document.querySelector("[data-node-id]")) {
    setInterval(refresh, 5000);
  }

  /* A running job reloads the page so its result appears without the operator
     clicking. The marker is only emitted while something is in flight, so an
     idle page never refreshes under them, and with scripting off the page
     still reports the outcome on the next manual load. */
  var poll = document.querySelector("[data-job-poll]");
  if (poll) {
    setTimeout(function () { window.location.reload(); },
      parseInt(poll.getAttribute("data-job-poll"), 10) || 5000);
  }
})();
"#;
