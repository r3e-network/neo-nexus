//! Headless containment verification: does the workbench paint inside itself?
//!
//! `tests/ui_geometry.rs` measures each panel's **clip rect**, which is the
//! region egui allows a panel to draw in. That is not the same as the region a
//! panel's content actually asks for, and the difference hid three real layout
//! faults at once: inventory rows grew wider with every item until the list
//! shoved the central workspace into a 271pt slot; the status bar laid out 38pt
//! of content inside a fixed 28pt strip; and the inspector stacked roughly
//! 1300pt of facts into a 725pt column so its lifecycle buttons were painted
//! where no one could see them.
//!
//! Every one of those passed the clip-rect contract. This one renders real
//! frames and asserts that painted geometry stays inside the region that will
//! actually be shown, so a container that silently truncates its own content
//! fails the build instead of shipping.

#[path = "ui_overflow/contracts.rs"]
mod contracts;
#[path = "ui_overflow/harness.rs"]
mod harness;
#[path = "ui_overflow/sub_tabs.rs"]
mod sub_tabs;
#[path = "ui_overflow/surfaces.rs"]
mod surfaces;
