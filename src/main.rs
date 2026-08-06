#![allow(warnings)]
// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;
mod fetcher;
use crate::fetcher::*;
mod error_handler;

fn main() {
    match run_command(String::from("which"), vec!["unity"]) {
        Some(path) =>  {
            match run_command(String::from(path), vec!["p", "list", "--json"]) {
                Some(output) => {
                    let mut i = 0;

                    while i < 20 {
                        match fetch_project(&output, i) {
                            Some(val) => println!("{:?}", val),
                            None => ()
                        }
                        i += 1;
                    }
                },
                None => println!("An error occured")
            }
        },
        None => ()
    }
}
