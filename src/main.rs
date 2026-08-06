#![allow(warnings)]
// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod fetcher;
use crate::fetcher::*;
mod error_handler;

fn main() {
    match fetch_projects() {
        Some(projects) => {
            for p in projects {
                println!("{:?}", p);
            }
        },
        None => eprintln!("Fetch failed!")
    }
}
