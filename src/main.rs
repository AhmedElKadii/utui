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

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let mut project_names: Vec<String> = Vec::new();
    let mut project_datas: Vec<ProjectData> = Vec::new();

    match get_projects() {
        Some(projects) => {
            project_names = projects.iter().map(|p| p.name.clone() as String).collect();
            project_datas = projects;
        },
        None => ()
    }

    let mut list_state = ListState::default().with_selected(Some(0));
    ratatui::run(|terminal| {
        loop {
            terminal.draw(|frame| render(frame, &mut list_state, project_names.clone()))?;
            if let Some(key) = event::read()?.as_key_press_event() {
                match key.code {
                    KeyCode::Char('j') | KeyCode::Down => list_state.select_next(),
                    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => list_state.select_next(),
                    KeyCode::Char('k') | KeyCode::Up => list_state.select_previous(),
                    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => list_state.select_previous(),
                    KeyCode::Char('d') => {
                        match list_state.selected_mut() {
                            Some(i) => {
                                match project_datas.get(i.clone()) {
                                    Some(pd) => {
                                        delete_project(pd, true);
                                        match get_projects() {
                                            Some(projects) => {
                                                project_names = projects.iter().map(|p| p.name.clone() as String).collect();
                                                project_datas = projects;
                                            },
                                            None => ()
                                        }
                                    },
                                    None => ()
                                }
                            },
                            None => ()
                        }
                    },
                    KeyCode::Char('o') => {
                        match list_state.selected_mut() {
                            Some(i) => {
                                match project_datas.get(i.clone()) {
                                    Some(pd) => open_project(pd),
                                    None => ()
                                }
                            },
                            None => ()
                        }
                    },
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    _ => {}
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
