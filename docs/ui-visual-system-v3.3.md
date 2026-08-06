# NeoNexus UI — v3.3 Visual System

| Field | Value |
|-------|-------|
| **Document** | Visual system (v3.3) |
| **Supersedes** | The colour, elevation and selection sections of `docs/ui-system-redesign-v3.1.md` |
| **Unchanged from v3.1** | Information architecture, type scale, 4pt spacing grid, fixed-panel model, component inventory |
| **Stack** | Pure Rust · eframe/egui 0.33 |

---

## What changed and why

v3.1 shipped a cool-grey, indigo-accented workbench where depth came from drop
shadows and selection came from solid accent fills. v3.3 keeps the whole
information architecture and replaces the surface language:

| | v3.1 | v3.3 |
|---|---|---|
| Surfaces | Cool grey (`236,236,240` … `255,255,255`) | Warm paper (`244,241,236` … `255,255,255`) |
| Accent | Indigo `88,86,214` | Coral `195,74,44` |
| Elevation | Drop shadow on cards | Flat: fill tier + hairline, **no shadow anywhere** |
| Selection | Solid accent block, white label | Accent **wash** + accent label + 3pt bar |
| Active toggle | Accent-filled segment | Raised card-coloured pill in a recessed track |
| Dividers | `ui.separator()` | Explicit `divider::hr` / `divider::vr` |

The goal is a calm, high-legibility operations surface: quiet paper tones, one
warm accent used sparingly, and crisp edges instead of soft shadows.

---

## Colour roles

Defined once in `src/app/theme/palette.rs` and read through the zero-argument
accessors in `src/app/theme.rs`. **Never reach past the accessors.**

| Role | Light | Dark | Use |
|---|---|---|---|
| `accent` | 195,74,44 | 203,76,40 | Filled primary buttons only |
| `accent_hover` | 168,62,35 | 224,92,54 | Hover/press on accent fills |
| `accent_text` | 176,66,31 | 240,138,98 | The accent **as glyphs**: selected nav labels, active segments, links |
| `accent_wash` | 251,237,231 | 58,37,28 | Selection tint: nav rows, list rows, pressed controls |
| `on_accent` | white | white | Labels on an accent fill |
| `text` | 28,25,23 | 245,242,238 | Body and headings |
| `muted_text` | 111,104,98 | 168,160,153 | Captions, secondary copy |
| `window_fill` | 244,241,236 | 18,17,16 | Central workspace canvas |
| `panel_fill` | 250,248,245 | 28,26,24 | Chrome (sidebar, header, status, side columns) |
| `card_fill` | 255,255,255 | 46,42,38 | Cards, list rows, raised pills |
| `faint_fill` | 241,237,231 | 55,50,45 | Recessed tracks, hover wash, inset surfaces |
| `border` | 230,225,217 | 69,63,57 | Hairlines |
| `border_strong` | 210,203,192 | 96,88,79 | Hover/focus edges, the active-pill edge |
| `status_running` | 22,124,69 | 74,222,128 | Success |
| `status_starting` | 180,83,9 | 251,191,36 | Warning |
| `status_stopped` | 122,114,106 | 154,146,138 | Neutral offline |
| `status_error` | 179,38,30 | 255,122,107 | Danger, destructive confirm fill |
| `info` | 16,101,127 | 86,199,232 | Info, network chips |

### The one rule that is easy to get wrong

**`accent()` is a fill. `accent_text()` is a glyph colour.**

The fill accent is deliberately dark enough that white button labels clear WCAG
AA (4.6:1). That same colour used as *text on paper* is legible but tight, and
in dark mode it is far too dark. `accent_text()` is the tuned counterpart:
darker in light mode, much brighter in dark mode.

Nothing in the test suite can catch `accent()` used as a text colour, so it is
a review item. `tests/unit/app/theme/tests.rs` enforces the contrast floors that
constrain the palette:

- body text ≥ 4.5:1 on every surface tier
- `on_accent` ≥ 4.5:1 on `accent`
- `muted_text` ≥ 4.5:1 on cards
- status hues ≥ 3.0:1 on cards
- dark tiers strictly darker than light tiers

A brighter, more saturated coral was tried first and rejected: it gives white
labels only 3.3:1.

---

## Elevation

There is no shadow anywhere in the workbench. `visuals.window_shadow` and
`visuals.popup_shadow` are `Shadow::NONE`, and `card_frame` carries a hairline
instead. Depth is entirely:

1. **Fill tier** — `window_fill` < `panel_fill` < `card_fill`, plus `faint_fill`
   as the *recessed* tone below all three.
2. **Hairline** — `theme::hairline()` on every card, panel and inset edge.

The direct consequence: **a card-coloured surface nested inside a card is
invisible.** Anything nested (form groups, busy strips, inset detail panels,
segmented tracks, chip tracks) fills `theme::track_surface()`, not
`card_surface()`.

Radii: 12 for cards and insets (`layout::CARD_CORNER_RADIUS`), 10 for tracks and
list rows, 9 for buttons and general widgets, 7 for segments and chips, 8 for
status badges.

---

## Selection

The workbench never paints a solid accent block behind body copy.

| Surface | Idle | Selected |
|---|---|---|
| Sidebar nav row | transparent | `accent_wash` + `accent_text` label + 3pt `accent` bar painted **inside** the row rect |
| List row | `card_fill` + hairline | `accent_wash` + 1pt `accent` hairline |
| Segment / filter chip | transparent on a `faint_fill` track | `card_fill` pill + `border_strong` hairline + `accent_text` label |
| `selectable_label` | — | `visuals.selection.bg_fill` = `accent_wash` |

The nav bar must be painted inside `response.rect`. Allocating it as a sibling
widget changes the row pitch and fails
`tests/ui_typography.rs::sidebar_navigation_rows_sit_on_a_consistent_baseline_grid`,
which pins the 44pt grid (`nav_row_height` 34 + `item_spacing.y` 8 + 2).

---

## Typography

The scale is unchanged (11 / 12 / 13 / 14 / 17 / 24) and enforced by
`tests/ui_typography.rs`. Two things changed:

1. **Headings state their colour.** `page_title`, `section_title` and
   `metric_value` set `.color(theme::text())` explicitly. Left to `.strong()`
   alone, egui resolves the colour from `Visuals::strong_text_color()`, which is
   the *pressed widget* foreground — a role that has nothing to do with page
   copy. See the regression note below.
2. **`theme::caption`** joins `label_caption`. `label_caption` force-uppercases
   its input, which mangles mixed-case field names ("WebSocket", "Max delay");
   `caption` keeps the caller's casing at the same 11pt muted treatment.

### Regression: the invisible-headings bug

`style.rs` set `widgets.active.fg_stroke` to `on_accent` (white) so that pressed
accent-filled buttons had white labels. Because egui derives
`strong_text_color()` from that same field, **every `.strong()` run in the app
rendered white** — page titles, section titles, metric values, and every status
bar value were invisible on the light theme. No contract caught it: the type
scale test checks sizes, not colours.

The fix is structural, not a patch: the pressed state is now a coral *wash* with
normal-strength text (`widgets.active.bg_fill = accent_wash`,
`fg_stroke = text`), which both matches the tinted-selection language and keeps
bold copy readable. Controls that genuinely need a white label on an accent fill
set it explicitly.

---

## Responsive layout

The design window is 1280pt and the workbench has up to four columns: sidebar
(212, fixed) + inventory (252, 220–300) + central workspace + inspector (304,
280–360). With everything open the workspace is roughly **450pt**. Pages must
adapt rather than assume.

- `widgets::fits_side_by_side(width)` — true when a column can hold two
  `MIN_COLUMN_WIDTH` (306pt) panes plus the gap. Used by the overview and by
  Node Studio to drop their second pane.
- `widgets::fits_side_by_side_at(width, min)` — the same test at a denser
  minimum, for form field groups that read fine narrower than a full pane.
- `widgets::metric_row` reflows and *balances*: four tiles in a three-wide space
  become 2 + 2, not 3 + 1.
- `widgets::segmented_control` uses equal columns only while every label fits at
  its real rendered width, and otherwise wraps segments at their natural widths.
  Six equal columns in a narrow workspace produced "Read…" and "Jour…".
- When the inspector is open, surfaces that would duplicate it drop their own
  copy — the overview's "Current selection" pane and Node Studio's "Selected
  node" pane both defer to it rather than competing for width.

### Sizing rules that prevent overflow

Three recurring mistakes caused every containment fault found in this pass:

1. **Measuring width outside a `Frame` and applying it inside.** The frame's
   inner margin is added on top, so the result is `2 × margin` too wide — and
   because that widens the parent, the *next* sibling is wider again. The
   inventory list fanned out 18pt per row until it had squeezed the central
   workspace to 271pt. Always size against `ui.available_width()` **inside** the
   frame.
2. **Double-counting item spacing in a two-pane `ui.horizontal`.** egui adds
   `item_spacing.x` around each allocated pane *and* around an explicit spacer.
   Set `ui.spacing_mut().item_spacing.x = 0.0` when the panes were sized to an
   exact split.
3. **Nesting `ui.horizontal` inside a fixed-height strip.** Each nested
   horizontal claims a full `interact_size.y`. The 28pt status bar was laying
   out 38pt of content. Lower `interact_size.y` for the strip and keep the row
   flat.

`tests/ui_overflow.rs` now fails the build on all three. `tests/ui_geometry.rs`
cannot: it measures each panel's **clip rect**, which is what egui allows a
panel to draw in, not what its content asked for.

---

## Component notes

| Widget | Notes |
|---|---|
| `card(ui, title, trailing, body)` | Flat titled card that **hugs** its content height, so cards stack. `layout::panel` is the stretch variant and delegates to the same chrome. |
| `inset_card(ui, body)` | Recessed detail surface for content nested in a card. Replaced six byte-identical hand-rolled frames. |
| `metric_grid(ui, pairs)` | Two-column label/value cells with hairline row rules. Not `Grid::striped` — banding reads as a data table, hairlines read as a spec sheet. |
| `divider::hr` / `divider::vr` | Explicit rules. `ui.separator()` auto-orients and claims a full `interact_size`, which is why the status bar overflowed. |
| `segmented_control` | Recessed track, raised active pill, reflows (see above). |
| `chip_pill` | The same track. Wraps its chips — a five-way status filter is wider than the inventory column. |
| `callout` | Body washed with the kind hue at low alpha, 3pt rail (`RAIL_WIDTH`, shared with the nav marker). |
| `primary_button` | Applies the accent through *widget visuals*, not `Button::fill`, so it actually responds to the pointer. Exactly one per region. |

---

## Verifying

`make verify` and CI now run the headless UI contracts. They render real frames
against a headless `egui::Context`, so they need no window and no screen-capture
permission:

```
make ui-contracts
```

For a look at the actual pixels — including glyphs, hairlines and rounded
corners — the opt-in rasterizer tessellates a real frame and samples egui's own
font atlas:

```
cargo test --release --test ui_visual_truth -- --ignored --nocapture
```

It writes `/tmp/neonexus_truth_<view>_<theme>.png` for every primary view in
both themes, from a seeded five-node fleet.

---

## Known limitation

Several surfaces still paint **taller** than their fixed panel, so content past
the panel edge is not visible: the Settings body below the active-policy block,
and the inspector's Overview actions. `tests/ui_overflow.rs` deliberately checks
horizontal containment only — a vertical check fails broadly today.

Fixing it is an information-architecture change, not a styling one: each
over-long surface needs paging or sectioning the way the inspector's
Overview / Paths / Process switcher already does. That switcher is the pattern
to copy.

---

## Containment: the rule that replaced "it looked fine"

The workbench has no scrolling anywhere. A surface that lays out more than its
panel holds does not become scrollable — egui culls the widgets that fall
entirely outside the clip, so the surplus is **silently dropped**. It is not
painted over other content, which is why it survived so long: every screenshot
looked plausible and every clip-rect assertion passed.

`tests/ui_overflow.rs` renders real frames for all six primary destinations,
with the inspector open and closed, and fails the build when painted geometry
leaves the region that will clip it — horizontally or vertically.

### The four sizing mistakes it catches

| Mistake | Symptom |
|---|---|
| Measuring width *outside* a frame and asking for it *inside* | Each row two margins too wide; the parent grows; the next row grows again |
| An absolute floor on a proportional split | `clamp(h * 0.48, 180, 280)` forces 368pt into a 200pt panel |
| A fixed page size for a list | Rows past the fold are laid out and dropped |
| Measuring text in `TextStyle::Body` | The app draws at 13pt; the fit check reads a different size and labels wrap mid-word |

### The four rules that follow

1. **Page sizes come from height.** `paging::rows_that_fit(available, row, chrome)`,
   with the old constant kept as an upper bound.
2. **Columns reflow, they do not shrink.** `widgets::columns_that_fit` wraps onto
   another row rather than squeezing a labelled field below the width of its label.
3. **Chrome earns its height.** A callout that reports nothing is wrong, a caption
   that repeats its own eyebrow, and a panel that restates the column beside it
   are all height taken from the content the operator came for.
4. **Chrome appears only where it applies.** The node inspector shows on node
   surfaces; on Runtimes, Network and Settings it is 332pt saying nothing.

## Grouping: surfaces are organised by what they act on

| Surface | Owns |
|---|---|
| **Nodes** › Studio · Config · Roles · Plugins · Logs · Health | Everything scoped to one node, in the order an operator works |
| **Runtimes** › Install · Catalog · Installed · Applied · Fast Sync | Binaries and snapshots on disk |
| **Network** › Remote · Private Net · Wallets | Topology and reach, plus the keys that sign for both |
| **Operations** | Fleet-wide readiness, ports, safety, journal |
| **Settings** | Policy: watchdog, upgrades, monitors, alerts, storage, release |

Role presets moved from Network to Nodes: a role is applied to one node, so it
belongs in that node's workspace. Private-network planning moved the other way —
a committee topology is not a property of any single node.
