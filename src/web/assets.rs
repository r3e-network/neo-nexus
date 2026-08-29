//! Embedded stylesheet and script. The source-purity gate keeps `.css`/`.js`
//! files out of the repository, so the browser assets live here as string
//! constants and ship inside the single binary. The script only polls the
//! fleet API and swaps status badges — every control works without JavaScript.

pub const CSS: &str = r#"
:root {
  --bg: #14161a; --panel: #1c1f26; --panel-2: #232731; --line: #2e3340;
  --text: #e8eaf0; --muted: #9aa1b2; --accent: #e0604d;
  --ok: #4cc38a; --warn: #d9a13b; --bad: #e0564d; --idle: #6b7280;
}
* { box-sizing: border-box; }
body { margin: 0; background: var(--bg); color: var(--text);
  font: 14px/1.5 "Segoe UI", system-ui, sans-serif; }
.shell { display: flex; min-height: 100vh; }
.sidebar { width: 200px; flex: 0 0 200px; background: var(--panel);
  border-right: 1px solid var(--line); padding: 16px 12px; display: flex;
  flex-direction: column; gap: 4px; }
.brand { font-weight: 700; font-size: 16px; padding: 4px 10px 14px; letter-spacing: .4px; }
.nav-item { display: block; padding: 8px 10px; border-radius: 8px;
  color: var(--muted); text-decoration: none; border: 0; background: none;
  width: 100%; text-align: left; font: inherit; cursor: pointer; }
.nav-item:hover { background: var(--panel-2); color: var(--text); }
.nav-item.current { background: var(--panel-2); color: var(--text); }
.nav-group { margin-bottom: 10px; }
.nav-title { color: var(--muted); font-size: 11px; text-transform: uppercase;
  letter-spacing: .7px; padding: 10px 10px 4px; }
.filters { display: flex; gap: 10px; flex-wrap: wrap; align-items: flex-end;
  margin-bottom: 14px; }
.field { display: flex; flex-direction: column; gap: 4px; font-size: 12px;
  color: var(--muted); }
.field input, .field select { background: var(--bg); color: var(--text);
  border: 1px solid var(--line); border-radius: 8px; padding: 6px 10px;
  font: inherit; min-width: 150px; }
.logout { margin-top: auto; color: var(--muted); }
.content { flex: 1; padding: 24px 28px; min-width: 0; }
h1 { font-size: 20px; margin: 0 0 18px; }
h2 { font-size: 15px; margin: 22px 0 10px; color: var(--muted);
  text-transform: uppercase; letter-spacing: .6px; }
.cards { display: flex; gap: 12px; flex-wrap: wrap; margin-bottom: 8px; }
.card { background: var(--panel); border: 1px solid var(--line);
  border-radius: 10px; padding: 14px 18px; min-width: 130px; }
.card .num { font-size: 24px; font-weight: 700; }
.card .lbl { color: var(--muted); font-size: 12px; }
table { border-collapse: collapse; width: 100%; background: var(--panel);
  border: 1px solid var(--line); border-radius: 10px; overflow: hidden; }
th, td { text-align: left; padding: 9px 12px; border-bottom: 1px solid var(--line); }
th { color: var(--muted); font-size: 12px; text-transform: uppercase;
  letter-spacing: .5px; background: var(--panel-2); }
tr:last-child td { border-bottom: 0; }
a { color: var(--text); }
.badge { display: inline-block; padding: 2px 10px; border-radius: 999px;
  font-size: 12px; font-weight: 600; }
.badge.running { background: rgba(76,195,138,.15); color: var(--ok); }
.badge.starting { background: rgba(217,161,59,.15); color: var(--warn); }
.badge.error { background: rgba(224,86,77,.15); color: var(--bad); }
.badge.stopped { background: rgba(107,114,128,.15); color: var(--idle); }
.flash { background: var(--panel-2); border: 1px solid var(--line);
  border-left: 3px solid var(--accent); border-radius: 8px;
  padding: 10px 14px; margin-bottom: 16px; }
.actions { display: flex; gap: 8px; }
button, .btn { background: var(--panel-2); color: var(--text);
  border: 1px solid var(--line); border-radius: 8px; padding: 6px 14px;
  font: inherit; cursor: pointer; text-decoration: none; display: inline-block; }
button:hover, .btn:hover { border-color: var(--accent); }
button.primary { background: var(--accent); border-color: var(--accent); color: #fff; }
.muted { color: var(--muted); }
.facts { max-width: 560px; }
pre { background: var(--panel); border: 1px solid var(--line);
  border-radius: 10px; padding: 14px 16px; overflow-x: auto; }
.login-wrap { display: flex; align-items: center; justify-content: center;
  min-height: 100vh; }
.login-card { background: var(--panel); border: 1px solid var(--line);
  border-radius: 12px; padding: 28px 32px; width: 340px; }
.login-card h1 { text-align: center; }
.login-card input { width: 100%; margin: 12px 0; padding: 9px 12px;
  background: var(--bg); color: var(--text); border: 1px solid var(--line);
  border-radius: 8px; font: inherit; }
.login-card .primary { width: 100%; }
.err { color: var(--bad); font-size: 13px; }
"#;

pub const SCRIPT: &str = r#"
(function () {
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
})();
"#;
