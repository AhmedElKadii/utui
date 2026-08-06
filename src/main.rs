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
    match fetch_projects() {
        Some(projects) => {
            for p in &projects {
                println!("{:?}", p);
            }

            let mut choice: String = String::new();

            io::stdin()
                .read_line(&mut choice)
                .expect("Failed to read line");

            match projects.get(choice.trim().parse::<usize>().unwrap()) {
                Some(p) => open_project(p),
                None => ()
            }
        },
        None => eprintln!("Fetch failed!")
    }
}
