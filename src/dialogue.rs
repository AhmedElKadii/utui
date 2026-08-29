use std::time::Instant;

use crate::project::Project;

#[derive(Clone, Default, PartialEq)]
pub enum Dialogue {
    #[default]
    None,
    DeleteConfirm { with_dir: bool },
    Input,
    Error(String),
    Info(String),
    TimedInfo(String, Instant),
    Confirm(String)
}

#[derive(Default, PartialEq)]
pub enum DialogueSelection {
    #[default]
    None,
    Ok,
    Cancel,
}

#[derive(Default)]
pub struct DialogueState {
    pub current: Dialogue,
    pub return_to: Dialogue,
    pub selection: DialogueSelection,
    pub selected_project: Option<Project>,
}

impl DialogueState {
    pub fn close(&mut self) {
        self.current = Dialogue::None;
        self.selected_project = None;
        self.selection = DialogueSelection::Ok;
    }

    pub fn show_error(&mut self, message: impl Into<String>) {
        self.current = Dialogue::Error(message.into());
        self.selection = DialogueSelection::Ok;
        self.selected_project = None;
    }
}
