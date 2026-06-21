use anyhow::Result;

mod actions;
mod output;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CliAction {
    Print(String),
    PrintWithExitCode { text: String, exit_code: i32 },
}

pub fn action_from_args<I, S>(args: I) -> Result<CliAction>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_string())
        .collect::<Vec<_>>();
    actions::action_from_args_vec(&args)
}

pub fn version_text() -> String {
    actions::version_text()
}

pub fn help_text() -> String {
    actions::help_text()
}

pub fn self_check_text() -> Result<String> {
    actions::self_check_text()
}
