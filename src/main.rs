#![allow(warnings)]
use std::io;
use arboard::Clipboard;
use color_eyre::eyre::Result;
use crossterm::event::{KeyEventKind};
use crossterm::event::{ self, KeyCode, KeyModifiers };
use ratatui::widgets::ListState;

// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod crud;
use crate::crud::*;
mod error_handler;
mod ui;
use crate::input::{InputHandler, InputStep};
use crate::ui::*;
mod input;

#[derive(Default, PartialEq)]
enum Dialogues {
    #[default]
    NULL,
    DELETE_CONFIRM(bool),
    INPUT,
    ERROR(String)
}

#[derive(Default, PartialEq)]
enum DialogueSelection {
    NULL,
    #[default]
    OK,
    CANCEL
}

#[derive(Default)]
struct DialogueState {
    current_dialogue: Dialogues,
    selection: DialogueSelection,
    selected_project: Option<ProjectData>
}

// TODO: use these inside of some APP instance to clean stuff up a bit
// struct AppState
// struct AppData

#[derive(Default)]
struct AppState {
    list_state: ListState,
    selected_index: Option<usize>,
    input_handler: InputHandler,
    dialogue_state: DialogueState,
    list_items: Vec<String>,
    list_items_buffer: Vec<String>,
    project_data: Vec<ProjectData>,
    editor_versions: Option<Vec<String>>,
    templates: Option<Vec<TemplateData>>,
    list_line_offset: usize,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut app_state = AppState::default();
    app_state.list_state = ListState::default().with_selected(Some(0));
    app_state.input_handler = InputHandler::new();

    refresh(&mut app_state);

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match app_state.dialogue_state.current_dialogue {
                    Dialogues::NULL => {
                        // main loop
                        match key.code {
                            KeyCode::Char('j') | KeyCode::Down => {
                                change_list(&mut app_state, true);
                            },
                            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                change_list(&mut app_state, true);
                            },
                            KeyCode::Char('k') | KeyCode::Up => {
                                change_list(&mut app_state, false);
                            },
                            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                change_list(&mut app_state, false);
                            },
                            KeyCode::Enter => expand_project(&mut app_state),
                            KeyCode::Char('D') | KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::SHIFT) => open_delete_dialogue(&mut app_state, true),
                            KeyCode::Char('d') => open_delete_dialogue(&mut app_state, false),
                            KeyCode::Char('o') => proj_open(&mut app_state),
                            KeyCode::Char('c') => open_proj_create_dialogue(&mut app_state),
                            KeyCode::Esc => collapse_project(&mut app_state),
                            KeyCode::Char('q') => break Ok(()),
                            _ => ()
                        }
                    },
                    Dialogues::INPUT => {
                        match key.code {
                            KeyCode::Enter => execute_selection(&mut app_state),
                            KeyCode::Tab if app_state.input_handler.step == InputStep::Path => append_list_item(&mut app_state),
                            KeyCode::Char(to_insert) if !key.modifiers.contains(KeyModifiers::CONTROL) => app_state.input_handler.enter_char(to_insert),
                            KeyCode::Backspace => app_state.input_handler.delete_char(),
                            KeyCode::Left => app_state.input_handler.move_cursor_left(),
                            KeyCode::Right => app_state.input_handler.move_cursor_right(),
                            KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                change_list(&mut app_state, true);
                            },
                            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                change_list(&mut app_state, false);
                            },
                            KeyCode::Char('v') if (key.modifiers.contains(KeyModifiers::CONTROL) ||
                                 key.modifiers.contains(KeyModifiers::META)) => {
                                let mut clipboard = Clipboard::new().unwrap();
                                app_state.input_handler.input = clipboard.get_text().unwrap();
                                app_state.input_handler.character_index = app_state.input_handler.input.len();
                            },
                            KeyCode::Esc => {
                                app_state.dialogue_state.selection = DialogueSelection::CANCEL;
                                execute_selection(&mut app_state);
                            },
                            _ => {}
                        }
                    },
                    _ => {
                        // dialogue sub-loop
                        match key.code {
                            KeyCode::Char('l') | KeyCode::Right => app_state.dialogue_state.selection = DialogueSelection::CANCEL,
                            KeyCode::Char('h') | KeyCode::Left => app_state.dialogue_state.selection = DialogueSelection::OK,
                            KeyCode::Esc => {
                                app_state.dialogue_state.selection = DialogueSelection::CANCEL;
                                execute_selection(&mut app_state);
                            },
                            KeyCode::Enter => execute_selection(&mut app_state),
                            KeyCode::Char('q') => break Ok(()),
                            _ => ()
                        }
                    }
                }
            }
        }
    })
}

