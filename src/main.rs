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
    // match get_projects() {
    //     Some(projects) => {
    //         let mut i = 0;
    //
    //         for p in &projects {
    //             println!("{}: {:?}", i, p);
    //             i += 1;
    //         }
    //
    //         let mut choice: String = String::new();
    //
    //         io::stdin()
    //             .read_line(&mut choice)
    //             .expect("Failed to read line");
    //
    //         match projects.get(choice.trim().parse::<usize>().unwrap()) {
    //             Some(p) => delete_project(p, true),
    //             None => ()
    //         }
    //     },
    //     None => eprintln!("Fetch failed!")
    // }
    
    match get_editors() {
        Some(editors) => {
            let mut i = 0;

            for e in &editors {
                println!("{}: {}", i, e);
                i += 1;
            }
        },
        None => eprintln!("Fetch failed!")
    }
}
