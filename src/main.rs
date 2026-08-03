#![allow(warnings)]
mod config;
// use crate::config::*;

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
}
