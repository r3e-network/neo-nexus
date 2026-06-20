mod health;
mod lifecycle;
mod preflight;

pub(in crate::private_network) use health::render_windows_health_script;
pub(in crate::private_network) use lifecycle::{
    render_windows_start_script, render_windows_stop_script,
};
pub(in crate::private_network) use preflight::render_windows_preflight_script;
