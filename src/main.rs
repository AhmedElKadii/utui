#![allow(warnings)]
// mod config;
// use crate::config::*;
mod commander;
use crate::commander::*;

use chrono::{DateTime, Utc};

struct ProjectData {
    git_tracked: bool,
    name: String,
    path: String,
    editor_version: String,
    last_opened: DateTime<Utc>
}

impl ProjectData {
    fn create_project() -> ProjectData {
        return ProjectData { 
            git_tracked: false,
            name: String::from("DEFAULT"),
            path: String::from("HOME_DIR"),
            editor_version: String::from("EDITOR_VERSION"),
            last_opened: Utc::now()
        };
    }
}

fn main() {
    let mut p1: ProjectData = ProjectData::create_project();

    println!("Name: {}", p1.name);

    let unity_path = run_command(String::from("which"), vec!["unity"]);

    let output = run_command(String::from(unity_path), vec!["p", "list", "--json"]);

    println!("{}", output);
}
