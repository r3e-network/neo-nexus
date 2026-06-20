mod quote;
mod unix;
mod windows;

pub(in crate::private_network) use self::{
    quote::sh_command_tokens,
    unix::{
        render_unix_health_script, render_unix_preflight_script, render_unix_start_script,
        render_unix_stop_script,
    },
    windows::{
        render_windows_health_script, render_windows_preflight_script, render_windows_start_script,
        render_windows_stop_script,
    },
};
