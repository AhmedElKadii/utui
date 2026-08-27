use crate::project::Project;

#[derive(Default, PartialEq)]
pub enum Dialogue {
    #[default]
    None,
    DeleteConfirm { with_dir: bool },
    Input,
    Error(String),
}

#[derive(Default, PartialEq)]
pub enum DialogueSelection {
    None,
    #[default]
    Ok,
    Cancel,
}

#[derive(Default)]
pub struct DialogueState {
    pub current: Dialogue,
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
