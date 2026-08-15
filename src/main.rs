#![allow(warnings)]
use std::io;
use color_eyre::eyre::Result;
use crossterm::event;
use crossterm::event::{ KeyCode, KeyModifiers };
use ratatui::widgets::ListState;

// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod crud;
use crate::crud::*;
mod error_handler;
mod ui;
use crate::ui::*;

#[derive(Default, PartialEq)]
enum Dialogues {
    #[default]
    NULL,
    DELETE_CONFIRM(bool),
}

#[derive(Default, PartialEq)]
enum DialogueSelection {
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

#[derive(Default)]
struct AppState {
    list_state: ListState,
    dialogue_state: DialogueState,
    list_items: Vec<String>,
    project_data: Vec<ProjectData>
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut app_state = AppState::default();
    app_state.list_state = ListState::default().with_selected(Some(0));

    refresh(&mut app_state);

    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut app_state))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match app_state.dialogue_state.current_dialogue {
                    Dialogues::NULL => {
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
                            KeyCode::Char('q') => break Ok(()),
                            _ => ()
                        }
                    },
                    _ => {
                        match key.code {
                            KeyCode::Char('l') | KeyCode::Right => app_state.dialogue_state.selection = DialogueSelection::CANCEL,
                            KeyCode::Char('h') | KeyCode::Left => app_state.dialogue_state.selection = DialogueSelection::OK,
                            KeyCode::Esc => {
                                app_state.dialogue_state.selected_project = None;
                                app_state.dialogue_state.current_dialogue = Dialogues::NULL;
                            },
                            KeyCode::Enter => {
                                execute_selection(&mut app_state);
                                continue;
                            },
                            KeyCode::Char('q') => break Ok(()),
                            _ => ()
                        }
                    }
                }
            }
        }
    })
}

fn list_projs() {
    match get_projects() {
        Some(projects) => {
            let mut i = 0;

            for p in &projects {
                println!("{}: {:?}", i, p);
                i += 1;
            }
        },
        None => eprintln!("Fetch failed!")
    }
}

fn delete_proj() {
    match get_projects() {
        Some(projects) => {
            let mut i = 0;

            for p in &projects {
                println!("{}: {:?}", i, p);
                i += 1;
            }

            let mut choice: String = String::new();

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");

            match projects.get(choice.trim().parse::<usize>().unwrap()) {
                Some(p) => delete_project(p, true),
                None => ()
            }
        },
        None => eprintln!("Fetch failed!")
    }
}

fn create_proj() {
    let mut name = String::new();
    let mut path = String::new();
    let mut editor = String::new();
    let mut template = String::new();
    let mut is_ready = true;

    println!("Name: ");

    io::stdin()
        .read_line(&mut name)
        .expect("null");

    println!("Path: ");

    io::stdin()
        .read_line(&mut path)
        .expect("null");

    match get_editors() {
        Some(editors) => {
            let mut i = 0;

            for e in &editors {
                println!("{}: {:?}", i, e);
                i += 1;
            }

            let mut choice: String = String::new();

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");

            match editors.get(choice.trim().parse::<usize>().unwrap()) {
                Some(e) => editor = e.clone(),
                None => ()
            }
        },
        None => eprintln!("Fetch failed!")
    }

    match get_templates(editor.clone()) {
        Some(templates) => {
            let mut i = 0;

            for t in &templates {
                println!("{}: {:?}", i, t.display_name);
                i += 1;
            }

            let mut choice: String = String::new();

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");

            match templates.get(choice.trim().parse::<usize>().unwrap()) {
                Some(t) => {
                    template = t.name.clone();
                    is_ready = t.status == TemplateStatus::READY;
                },
                None => eprintln!("Failed to get template")
            }
        },
        None => eprintln!("Fetch failed!")
    }
    
    match create_project(name, editor, template, path, is_ready) {
        Ok((true, o)) => println!("{}", o),
        _ => eprintln!("An error occured")
    }
}
