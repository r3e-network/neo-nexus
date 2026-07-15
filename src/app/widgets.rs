mod badges;
mod callout;
mod controls;
mod filter_bar;
mod form;
mod layout;
mod node_row;
mod nodes;
mod page_header;
mod plugins;
mod segmented;
mod toolbar;

#[allow(unused_imports)] // status_dot/severity_color used by node_row and future tables
pub(super) use badges::{
    severity_badge, severity_color, status_badge, status_dot, text_badge,
};
pub(super) use callout::{callout, CalloutKind};
pub(super) use controls::{
    chip_pill, labeled_combo, labeled_text, pagination_bar, primary_button, secondary_button,
    secondary_button_enabled,
};
pub(super) use filter_bar::{filter_bar, filter_chip};
#[allow(unused_imports)] // form_section reserved for denser multi-card forms
pub(super) use form::{field_combo, field_text, form_group, form_section};
#[allow(unused_imports)]
pub(super) use layout::{
    card_frame, empty_state, empty_state_with_action, grid_header, metric_row, mini_stat, panel,
};
pub(super) use node_row::node_row;
pub(super) use nodes::{fact, fact_error, render_node_fact_sheet};
#[allow(unused_imports)] // Phase 2 multi-section pages
pub(super) use page_header::page_header;
pub(super) use plugins::plugin_enabled;
pub(super) use segmented::segmented_control;
pub(super) use toolbar::{toolbar, ToolbarAction};
