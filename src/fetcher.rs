use std::error::Error;
use serde_json::Value;

use crate::commander;
use crate::error_handler;
use chrono::prelude::*;

#[derive(Debug)]
pub struct ProjectData {
    pub git_tracked: bool,
    pub name: String,
    pub path: String,
    pub editor_version: String,
    pub last_opened: String
}

impl ProjectData {
    fn create_project() -> ProjectData {
        return ProjectData { 
            git_tracked: false,
            name: String::from("DEFAULT"),
            path: String::from("HOME_DIR"),
            editor_version: String::from("EDITOR_VERSION"),
            last_opened: String::from("")
        };
    }
}

fn load_json(json_str: &str) -> Result<Value, Box<dyn Error>> {
    if let json_value = serde_json::from_str(json_str)? {
        return Ok(json_value);
    }

    return Err(Box::new(error_handler::ParseFailErr("[SER01] failed to parse json.".into())));
}

fn is_tracked(path: &str) -> bool {
    match commander::run_command(String::from("ls"), vec!["-a", path]) {
        Some(output) => {
            return output.contains(".git");
        },
        None => {
            return false;
        }
    }
}

pub fn fetch_project(json_value: Option<Value>, index: usize) -> Option<ProjectData> {
    let mut project: ProjectData = ProjectData::create_project();

    let loaded_value = {
        match json_value {
            Some(val) => val,
            None => {
                match commander::run_command(String::from("which"), vec!["unity"]) {
                    Some(path) =>  {
                        match commander::run_command(String::from(path), vec!["p", "list", "--json"]) {
                            Some(output) => {
                                match load_json(&output) {
                                    Ok(val) => val,
                                    Err(e) => {
                                        eprintln!("Failed to load json");
                                        return None;
                                    }
                                }
                            },
                            None => {
                                eprintln!("Failed to run command");
                                return None;
                            }
                        }
                    },
                    None => {
                        eprintln!("Failed to run command");
                        return None;
                    }
                }
            }
        }
    };


    if loaded_value != Value::Null {
        match loaded_value.get("data").and_then(|v| v.as_array()) {
            Some(data) => {
                match data[index].get("title").and_then(|v| v.as_str()) {
                    Some(title) => project.name = String::from(title),
                    _ => (),
                }
                match data[index].get("path").and_then(|v| v.as_str()) {
                    Some(path) => project.path = String::from(path),
                    _ => (),
                }
                match commander::run_command(String::from("ls"), vec!["-a", &project.path]) {
                    Some(value) => (),
                    None => return None
                }
                match data[index].get("version").and_then(|v| v.as_str()) {
                    Some(version) => project.editor_version = String::from(version),
                    _ => (),
                }
                match is_tracked(&project.path) {
                    true => project.git_tracked = true,
                    _ => project.git_tracked = false,
                }
                match data[index].get("lastModified").and_then(|v| v.as_i64()) {
                    Some(last_modified) => {
                        let secs = last_modified / 1000;
                        let millis = (last_modified % 1000) as u32;

                        let timeModified: DateTime<Utc> = DateTime::from_timestamp(secs, millis * 1_000_000)
                            .expect("invalid timestamp");
                        let currentTime: DateTime<Utc> = Utc::now();

                        let diff = currentTime - timeModified;

                        if diff.num_days() == 0 {
                            if diff.num_hours() < 1 {
                                if diff.num_minutes() < 1 {
                                    project.last_opened = format!("{} second(s) ago.", diff.num_seconds().max(0));
                                }
                                else {
                                    project.last_opened = format!("{} minute(s) ago.", diff.num_minutes().max(0));
                                }
                            }
                            else {
                                project.last_opened = format!("{} hour(s) ago.", diff.num_hours().max(0));
                            }
                        }
                        else if diff.num_days() > 0 && diff.num_days() < 7 {
                            project.last_opened = format!("{} day(s) ago.", diff.num_days().max(0));
                        }
                        else if diff.num_days() == 7 {
                            project.last_opened = format!("{} week(s) ago.", diff.num_weeks().max(0));
                        }
                        else {
                            project.last_opened = timeModified.format("%d/%m/%Y").to_string();
                        }
                    },
                    _ => (),
                }
                return Some(project);
            }
            _ => (),
        }
    }
    return None;
}

pub fn fetch_projects() -> Option<Vec<ProjectData>> {
    let mut projects: Vec<ProjectData> = Vec::new();

    match commander::run_command(String::from("which"), vec!["unity"]) {
        Some(path) =>  {
            match commander::run_command(String::from(path), vec!["p", "list", "--json"]) {
                Some(output) => {
                    match load_json(&output) {
                        Ok(json_value) => {
                            match json_value.get("data").and_then(|v| v.as_array()) {
                                Some(arr) => {
                                    let mut i = 0;
                                    while i < arr.len() {
                                        match fetch_project(Some(json_value.clone()), i) {
                                            Some(p) => projects.push(p),
                                            None => ()
                                        }
                                        i += 1;
                                    }
                                    return Some(projects);
                                },
                                None => {
                                    eprintln!("Failed to fetch json array");
                                    return None;
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Failed to load json");
                            return None;
                        }
                    }
                },
                None => {
                    eprintln!("Failed to run command");
                    return None;
                }
            }
        },
        None => {
            eprintln!("Failed to run command");
            return None;
        }
    }
}
