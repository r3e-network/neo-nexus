#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagerCliOutput {
    pub(super) text: String,
    pub(super) exit_code: i32,
}

impl ManagerCliOutput {
    pub fn text_with_trailing_newline(&self) -> String {
        if self.text.ends_with('\n') {
            self.text.clone()
        } else {
            format!("{}\n", self.text)
        }
    }

    pub fn exit_code(&self) -> i32 {
        self.exit_code
    }

    pub fn should_exit_process(&self) -> bool {
        self.exit_code != 0
    }
}
