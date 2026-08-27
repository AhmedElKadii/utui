#![allow(warnings)]
use arboard::Clipboard;
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::widgets::ListState;
use rust_fuzzy_search::fuzzy_search_threshold;
use std::{default, fs};
use std::time::Duration;
use std::sync::Arc;

use crate::dialogue::{Dialogue, DialogueSelection, DialogueState};
use crate::error::AppError;
use crate::input::{InputHandler, InputStep};
use crate::project::Project;
use crate::template::Template;
use crate::unity::UnityCLI;
use crate::threading::AsyncTask;

#[derive(Default)]
pub struct Tasks {
    pub projects: Option<AsyncTask<Result<(Vec<String>, Vec<Project>), AppError>>>,
    pub editors: Option<AsyncTask<Result<(Vec<String>), AppError>>>,
    pub templates: Option<AsyncTask<Result<(Vec<String>, Vec<Template>), AppError>>>,
}

pub struct App {
    pub list_state: ListState,
    pub selected_index: Option<usize>,
    pub input: InputHandler,
    pub dialogue: DialogueState,
    pub list_items: Vec<String>,
    pub list_items_buffer: Vec<String>,
    pub projects: Vec<Project>,
    pub editor_versions: Option<Vec<String>>,
    pub templates: Option<Vec<Template>>,
    pub tick_counter: u64, 
    pub tasks: Tasks,
    unity: Option<Arc<UnityCLI>>
}

impl App {
    pub fn new() -> Self {
        Self {
            list_state: ListState::default().with_selected(Some(0)),
            selected_index: None,
            input: InputHandler::new(),
            dialogue: DialogueState::default(),
            list_items: Vec::new(),
            list_items_buffer: Vec::new(),
            projects: Vec::new(),
            editor_versions: None,
            templates: None,
            tasks: Tasks::default(),
            tick_counter: 0,
            unity: match UnityCLI::discover() {
                Ok(unity_instance) => {
                    Some(Arc::new(unity_instance))
                }
                Err(err) => {
                    eprintln!("Unity CLI not found: {}", err);
                    None
                }
            },
        }
    }

    pub fn run() -> color_eyre::Result<()> {
        let mut app = Self::new();
        app.refresh();

        let mut terminal = ratatui::init();

        let result = loop {
            terminal.draw(|frame| crate::ui::render(frame, &mut app))?;

            app.tick_counter = app.tick_counter.wrapping_add(1);

            if let Some(ref task) = app.tasks.projects {
                if let Some(result) = task.poll() {
                    app.tasks.projects = None;
                    match result {
                        Ok((names, projects)) => {
                            app.list_items = names;
                            app.projects = projects;
                            app.dialogue.current = Dialogue::None;
                        }
                        Err(err) => {
                            app.dialogue.current = Dialogue::Error(err.to_string());
                        }
                    }
                } else if matches!(app.dialogue.current, Dialogue::Info(_)) {
                    app.update_loading(String::from("Loading projects..."));
                }
            }

            if let Some(ref task) = app.tasks.editors {
                if let Some(result) = task.poll() {
                    app.tasks.editors = None;
                    match result {
                        Ok(editors) => {
                            if editors.is_empty() {
                                app.dialogue.current = Dialogue::Error("No editors available...".to_string());
                            } else {
                                app.editor_versions = Some(editors.clone());
                                app.list_items = editors;
                                app.dialogue.current = Dialogue::Input;
                            }
                        }
                        Err(err) => {
                            app.dialogue.current = Dialogue::Error(err.to_string());
                        }
                    }
                } else if matches!(app.dialogue.current, Dialogue::Info(_)) {
                    app.update_loading(String::from("Loading editor versions..."));
                }
            }

            if event::poll(Duration::from_millis(16))? {
                if let Some(key) = event::read()?.as_key_press_event() {
                    if app.handle_key(key) {
                        break Ok(());
                    }
                }
            }
        };

        ratatui::restore();
        result
    }

    pub fn update_loading(&mut self, message: String) {
        let spinner_frames = ["⠦", "⠖", "⠲", "⠴"];

        let index = (self.tick_counter / 8) % spinner_frames.len() as u64;
        let spinner = spinner_frames[index as usize];

        self.dialogue.current = Dialogue::Info(format!("{} {}", spinner, message));
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        match self.dialogue.current {
            Dialogue::None => self.handle_main_key(key),
            Dialogue::Input => self.handle_input_key(key),
            Dialogue::DeleteConfirm { .. } | 
                Dialogue::Error(_) |
                Dialogue::Info(_) => self.handle_dialogue_key(key),
        }
    }

    fn handle_main_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.move_selection(true),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(true)
            }
            KeyCode::Char('k') | KeyCode::Up => self.move_selection(false),
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(false)
            }
            KeyCode::Enter => self.toggle_project_details(),
            KeyCode::Char('D') | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.open_delete_dialogue(true)
            }
            KeyCode::Char('d') => self.open_delete_dialogue(false),
            KeyCode::Char('o') => self.open_selected_project(),
            KeyCode::Char('c') => self.open_create_dialogue(),
            KeyCode::Esc => self.collapse_project(),
            KeyCode::Char('q') => return true,
            _ => {}
        }
        false
    }

    fn handle_input_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Enter => self.execute_selection(),
            KeyCode::Tab if self.input.step == InputStep::Path => self.append_path_item(),
            KeyCode::Char(to_insert) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.input.enter_char(to_insert)
            }
            KeyCode::Backspace => self.input.delete_char(),
            KeyCode::Left => self.input.move_cursor_left(),
            KeyCode::Right => self.input.move_cursor_right(),
            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(true)
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.move_selection(false)
            }
            KeyCode::Char('v')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    || key.modifiers.contains(KeyModifiers::META) =>
            {
                self.paste_clipboard();
            }
            KeyCode::Esc => {
                self.dialogue.selection = DialogueSelection::Cancel;
                self.execute_selection();
            }
            _ => {}
        }
        false
    }

    fn handle_dialogue_key(&mut self, key: KeyEvent) -> bool {
        match key.code {
            KeyCode::Char('l') | KeyCode::Right => {
                self.dialogue.selection = DialogueSelection::Cancel
            }
            KeyCode::Char('h') | KeyCode::Left => self.dialogue.selection = DialogueSelection::Ok,
            KeyCode::Esc => {
                self.dialogue.selection = DialogueSelection::Cancel;
                self.execute_selection();
            }
            KeyCode::Enter if self.dialogue.selection != DialogueSelection::None => self.execute_selection(),
            KeyCode::Char('q') => return true,
            _ => {}
        }
        false
    }

    pub fn refresh(&mut self) {
        self.list_items.clear();
        self.projects.clear();

        if let Some(unity) = &self.unity {
            let uclone = unity.clone();

            self.tasks.projects = Some(AsyncTask::new(move || {
                uclone.list_projects().map(|projects| {
                    let names = projects.iter().map(|p| p.name.clone()).collect();
                    (names, projects)
                })
            }));
        }

        self.dialogue.close();
        self.list_state.select_first();
    }

    pub fn prepare_input_lists(&mut self) {
        if self.dialogue.current != Dialogue::Input {
            return;
        }

        match self.input.step {
            InputStep::Path => self.refresh_path_suggestions(),
            InputStep::Version => self.refresh_version_suggestions(),
            InputStep::Template => self.refresh_template_suggestions(),
            _ => {}
        }
    }

    fn refresh_path_suggestions(&mut self) {
        if self.input.value.is_empty() {
            self.input.set_text("/");
        }

        if self.input.value.ends_with('/') {
            self.list_items = dir_contents(&self.input.value);
            self.list_items
                .sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
            self.list_items_buffer = self.list_items.clone();
        } else {
            let query = self
                .input
                .value
                .rsplit_once('/')
                .map(|(_, text)| text)
                .unwrap_or("");
            self.list_items_buffer = fuzzy_filter_sorted(query, self.list_items.clone());
        }

        if self.list_state.selected().is_none() && self.list_items_buffer.len() == 1 {
            self.list_state.select_first();
        }
    }

    fn refresh_version_suggestions(&mut self) {
        if self.editor_versions.is_none() {
            if let Some(unity) = &self.unity {
                let uclone = unity.clone();

                self.dialogue.current = Dialogue::Info(String::new());
                self.tasks.editors = Some(AsyncTask::new(move || {
                    uclone.list_editors()
                }));
            }
        }

        self.list_items = fuzzy_filter_sorted(&self.input.value, self.list_items.clone());
    }

    fn refresh_template_suggestions(&mut self) {
        if self.templates.is_none() {
            let version = self
                .dialogue
                .selected_project
                .as_ref()
                .map(|project| project.editor_version.clone())
                .unwrap_or_default();
            self.templates = self
                .unity
                .as_ref()
                .and_then(|unity| unity.list_templates(&version).ok());
        }
    }

    pub fn template_labels(&self) -> Vec<String> {
        let labels = self
            .templates
            .as_ref()
            .map(|templates| templates.iter().map(Template::list_label).collect())
            .unwrap_or_default();
        fuzzy_filter_sorted(&self.input.value, labels)
    }

    fn finish_create(&mut self) {
        if let (Some(unity), Some(project)) = (&self.unity, &self.dialogue.selected_project) {
            if let Err(err) = unity.create_project(project) {
                self.dialogue.show_error(err.to_string());
                self.input.step = InputStep::Name;
                return;
            }
        }

        self.reset_after_dialogue();
        self.refresh();
    }

    pub fn execute_selection(&mut self) {
        match self.dialogue.current {
            Dialogue::DeleteConfirm { with_dir } => {
                if self.dialogue.selection == DialogueSelection::Ok {
                    self.delete_selected(with_dir);
                }
                self.reset_after_dialogue();
            }
            Dialogue::Input => {
                if self.dialogue.selection == DialogueSelection::Cancel {
                    self.reset_after_dialogue();
                    self.input.submit_message();
                    self.refresh();
                    return;
                }

                if self.input.value.is_empty() {
                    return;
                }

                if !self.advance_create_step() {
                    return;
                }
                self.input.submit_message();
                if self.input.step == InputStep::Complete {
                    self.finish_create();
                }
            }
            Dialogue::Error(_) | Dialogue::Info(_) => self.reset_after_dialogue(),
            Dialogue::None => {}
        }
    }

    fn advance_create_step(&mut self) -> bool {
        match self.input.step {
            InputStep::Name => {
                let name = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.name = name;
                }
                self.input.step = InputStep::Path;
                true
            }
            InputStep::Path => {
                let path = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.path = path;
                }
                self.input.step = InputStep::Version;
                true
            }
            InputStep::Version => {
                if !self.list_items.contains(&self.input.value) {
                    return false;
                }
                let version = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.editor_version = version;
                }
                self.input.step = InputStep::Template;
                true
            }
            InputStep::Template => {
                if let (Some(templates), Some(index)) =
                    (self.templates.as_ref(), self.list_state.selected())
                {
                    let names: Vec<String> = templates.iter().map(|t| t.name.clone()).collect();
                    if let Some(value) = names.get(index).cloned() {
                        self.input.set_text(value);
                        if !names.contains(&self.input.value) {
                            return false;
                        }
                    }
                }
                let template = self.input.value.clone();
                if let Some(project) = self.dialogue.selected_project.as_mut() {
                    project.template = template;
                }
                self.input.step = InputStep::Complete;
                true
            }
            _ => false,
        }
    }

    pub fn append_path_item(&mut self) {
        if self.list_state.selected().is_none() {
            self.list_state.select_first();
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(value) = self.list_items_buffer.get(index).cloned() else {
            return;
        };

        let prefix_end = self.input.value.rfind('/').unwrap_or(0);
        let mut path = self.input.value[..=prefix_end].to_string();
        path.push_str(&value);
        path.push('/');
        self.input.set_text(&path);
        self.list_state.select(None);
        self.list_items.clear();
    }

    pub fn move_selection(&mut self, next: bool) {
        if self.list_items.is_empty() {
            return;
        }

        if let Some(index) = self.list_state.selected() {
            let should_collapse = self.dialogue.current == Dialogue::None
                && if next {
                    index < self.list_items.len() - 1
                } else {
                    index > 0
                };
            if should_collapse {
                self.collapse_project();
            }
        }

        if next {
            self.list_state.select_next();
        } else {
            self.list_state.select_previous();
        }

        if self.input.step == InputStep::Version {
            if let (Some(versions), Some(index)) =
                (self.editor_versions.as_ref(), self.list_state.selected())
            {
                if let Some(value) = versions.get(index) {
                    self.input.set_text(value);
                }
            }
        }
    }

    fn open_create_dialogue(&mut self) {
        self.selected_index = self.list_state.selected();
        self.list_state.select(None);
        self.dialogue.current = Dialogue::Input;
        self.dialogue.selected_project = Some(Project::default());
        self.dialogue.selection = DialogueSelection::Ok;
        self.input.step = InputStep::Name;
        self.templates = None;
    }

    fn open_delete_dialogue(&mut self, with_dir: bool) {
        if self.list_items.is_empty() {
            return;
        }

        if let Some(index) = self.list_state.selected() {
            if let Some(project) = self.projects.get(index).cloned() {
                self.dialogue.current = Dialogue::DeleteConfirm { with_dir };
                self.dialogue.selected_project = Some(project);
                self.dialogue.selection = DialogueSelection::Cancel;
            }
        }

        self.selected_index = self.list_state.selected();
        self.list_state.select(None);
    }

    fn open_selected_project(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };
        let Some(project) = self.projects.get(index) else {
            return;
        };
        if let Some(unity) = &self.unity {
            let _ = unity.open_project(project);
        }
    }

    fn delete_selected(&mut self, with_dir: bool) {
        let Some(project) = self.dialogue.selected_project.clone() else {
            return;
        };
        if let Some(unity) = &self.unity {
            let _ = unity.delete_project(&project, with_dir);
        }
        self.refresh();
    }

    fn toggle_project_details(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };

        if self.list_items[index].contains('\n') {
            self.collapse_project();
            return;
        }

        if let Some(project) = self.projects.get(index) {
            self.list_items[index] = project.details_text();
        }
    }

    fn collapse_project(&mut self) {
        if self.list_items.is_empty() {
            return;
        }

        let Some(index) = self.list_state.selected() else {
            return;
        };

        if let Some(project) = self.projects.get(index) {
            self.list_items[index] = project.name.clone();
        }
    }

    fn reset_after_dialogue(&mut self) {
        self.list_state.select(self.selected_index);
        self.dialogue.close();
        self.input.step = InputStep::Name;
    }

    fn paste_clipboard(&mut self) {
        if let Ok(mut clipboard) = Clipboard::new() {
            if let Ok(text) = clipboard.get_text() {
                self.input.set_text(&text);
            }
        }
    }
}

fn dir_contents(dir_path: &str) -> Vec<String> {
    fs::read_dir(dir_path)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect()
}

pub fn fuzzy_filter_sorted(query: &str, list_items: Vec<String>) -> Vec<String> {
    if query.is_empty() {
        return list_items;
    }

    let refs: Vec<&str> = list_items.iter().map(String::as_str).collect();
    let mut results = fuzzy_search_threshold(query, &refs, 0.5);
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results
        .into_iter()
        .map(|(word, _)| word.to_string())
        .collect()
}
