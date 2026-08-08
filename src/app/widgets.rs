mod badges;
mod busy;
mod callout;
mod card;
mod columns;
mod confirm_bar;
mod controls;
mod divider;
mod filter_bar;
mod form;
mod inset_card;
mod layout;
mod list_row;
mod metric_grid;
mod node_row;
mod nodes;
mod page_chrome;
mod page_header;
mod plugins;
mod segmented;
mod toolbar;

#[allow(unused_imports)]
pub(super) use badges::{severity_badge, severity_color, status_badge, status_dot, text_badge};
pub(super) use busy::{busy_inline, loading_callout};
pub(super) use callout::{callout, CalloutKind, RAIL_WIDTH};
pub(super) use card::card;
#[allow(unused_imports)] // adopted surface-by-surface
pub(super) use columns::{
    columns_that_fit, fits_side_by_side, fits_side_by_side_at, MIN_COLUMN_WIDTH,
};
pub(super) use confirm_bar::confirm_bar;
pub(super) use controls::{
    chip_pill, labeled_combo, labeled_text, pagination_bar, primary_button, secondary_button,
    secondary_button_enabled,
};
#[allow(unused_imports)] // hr adopted surface-by-surface as separators retire
pub(super) use divider::{hr, hr_tight, vr};
pub(super) use filter_bar::{filter_bar, filter_chip};
#[allow(unused_imports)]
pub(super) use form::{field_combo, field_static, field_text, form_group, form_section};
#[allow(unused_imports)] // adopted surface-by-surface across the six hand-rolled insets
pub(super) use inset_card::inset_card;
#[allow(unused_imports)]
pub(super) use layout::{
    card_frame, empty_state, empty_state_with_action, grid_header, metric_row, mini_stat, panel,
};
pub(super) use list_row::list_row_frame;
#[allow(unused_imports)] // adopted by the inspector card stack
pub(super) use metric_grid::metric_grid;
pub(super) use node_row::node_row;
pub(super) use nodes::{fact, fact_error, render_node_fact_sheet};
pub(super) use page_chrome::page_chrome;
#[allow(unused_imports)] // superseded by page_chrome on primaries
pub(super) use page_header::page_header;
pub(super) use plugins::plugin_enabled;
// segmented_control is private to page_chrome; surfaces use page_chrome only.
pub(super) use toolbar::{toolbar, ToolbarAction};
