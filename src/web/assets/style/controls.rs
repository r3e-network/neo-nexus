//! Status badges, buttons, form controls and flash banners.
//!
//! One slice of the console stylesheet. Slices are concatenated in source
//! order by `super::css`; see `style.rs` for why the sheet is not one file.

pub(super) const RULES: &str = r#"/* Status Badges & Dots */
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

/* A flash is the outcome of the last control. Success reads in jade; failure
   reads in red, so the two are distinguishable at a glance rather than by
   reading the sentence. */
.flash { border-left-color: var(--jade); background: rgba(59, 209, 132, 0.06); }
.flash.danger { border-left-color: var(--red); background: rgba(244, 63, 94, 0.07); }
.flash.danger strong { color: var(--red); }

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

"#;
