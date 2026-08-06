use super::*;

/// How far a `node_row` paints past its nominal density slot in this column.
///
/// `list_row_frame` treats the slot as a frame *minimum*, not a clamp: the
/// Comfortable two-line anatomy (name/status over type/network/port) adds a
/// whole badge line, so a 44pt slot paints 86pt, while Compact's single line
/// only adds the frame's padding and hairline (40 → 42). Both measured in the
/// 252pt-wide inventory column at the 1280x820 design window. Sizing the page
/// from the bare slot is what left the last rows below the panel edge.
const ROW_OVERRUN_COMFORTABLE: f32 = 42.0;
const ROW_OVERRUN_COMPACT: f32 = 2.0;

/// The pagination bar is one row of `secondary_button`s, whose 30pt minimum
/// height is what sets it.
const PAGINATION_BAR_HEIGHT: f32 = 30.0;

impl NeoNexusApp {
    pub(in crate::app::views::shell::inventory) fn render_inventory_nodes(
        &mut self,
        ui: &mut egui::Ui,
    ) {
        let nodes = self.filtered_inventory_nodes();

        if self.fleet.nodes.is_empty() {
            if empty_state_with_action(
                ui,
                "No nodes",
                "Define the first local Neo runtime to start operating.",
                Some("New Node"),
            ) {
                self.session.selected_view = crate::app::view::View::Nodes;
            }
            return;
        }

        if nodes.is_empty() {
            empty_state(
                ui,
                "No matching nodes",
                "No local nodes match the current filter.",
            );
            return;
        }

        let row_height = inventory_row_height(self.session.density);
        let spacing = ui.spacing().item_spacing.y;
        // Measured here, before the pagination bar exists: the header, the
        // mini-stat block, the wrapped status chips and the search field are
        // drawn by sibling calls and are already out of `available_height`,
        // but the bar below belongs to this function and has to be reserved.
        let page_size = rows_that_fit(
            ui.available_height(),
            row_height + theme::XS + spacing,
            PAGINATION_BAR_HEIGHT + theme::SM + spacing,
        )
        .min(NODE_PAGE_SIZE);

        let total_pages = page_count(nodes.len(), page_size);
        self.fleet.node_page = self.fleet.node_page.min(total_pages - 1);
        pagination_bar(ui, &mut self.fleet.node_page, total_pages, nodes.len());
        ui.add_space(theme::SM);

        let start = self.fleet.node_page * page_size;
        let visible: Vec<_> = nodes.iter().skip(start).take(page_size).collect();
        let mut next_selection = self.fleet.selected_node.clone();

        for row in 0..page_size {
            if let Some(node) = visible.get(row) {
                let selected = next_selection.as_deref() == Some(node.id.as_str());
                if node_row(ui, node, selected, true, self.session.density) {
                    next_selection = Some(node.id.clone());
                }
                ui.add_space(theme::XS);
            } else {
                // Reserve row height so the list stays evenly paced when a page
                // is only partially filled (matches inventory node_row height).
                ui.add_space(row_height);
            }
        }

        self.select_fleet_node(next_selection);
    }
}

/// Height one inventory row actually paints, slot plus the overrun above.
fn inventory_row_height(density: theme::UiDensity) -> f32 {
    theme::DensityMetrics::for_density(density).list_row_compact
        + match density {
            theme::UiDensity::Comfortable => ROW_OVERRUN_COMFORTABLE,
            theme::UiDensity::Compact => ROW_OVERRUN_COMPACT,
        }
}
