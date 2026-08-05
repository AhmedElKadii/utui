#![allow(warnings)]
// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod fetcher;
use crate::fetcher::*;
mod error_handler;

fn main() {
    // let mut p1: ProjectData = ProjectData::create_project();
    //
    // println!("Name: {}", p1.name);
    //
    let unity_path = run_command(String::from("which"), vec!["unity"]);
    let output = run_command(String::from(unity_path), vec!["p", "list", "--json"]);

    let mut i = 0;

    while i < 30 {
        match fetch_project(&output, i) {
            Some(val) => println!("{:?}", val),
            None => println!("An error occurred")
        }
        i += 1;
    }
}
