#![allow(warnings)]
use std::io;

// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod crud;
use crate::crud::*;
mod error_handler;

fn main() {
    create_proj();
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
