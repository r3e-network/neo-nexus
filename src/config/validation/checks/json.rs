mod arrays;
mod plugins;
mod scalar;

pub(in crate::config::validation) use arrays::check_json_array_len_at_least;
pub(in crate::config::validation) use plugins::check_neo_cli_plugins;
pub(in crate::config::validation) use scalar::{
    check_json_bool, check_json_string, check_json_u16, check_json_u32, check_json_u8,
};
