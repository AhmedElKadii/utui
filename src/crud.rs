use std::error::Error;
use serde_json::Value;
use strum_macros::EnumString;
use std::str::FromStr;
use std::io;

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
            path: String::from("DEFAULT"),
            editor_version: String::from("DEFAULT"),
            last_opened: String::from("DEFAULT")
        };
    }
}

#[derive(Debug, EnumString)]
enum TemplateType { NULL, CORE, LEARNING, SAMPLE }

#[derive(Debug, EnumString)]
enum RenderPipeline { NULL, HDRP, URP, BUILT_IN }

#[derive(Debug, EnumString)]
enum BuildPlatform {
    NULL,
    IOS,
    ANDROID,
    LINUX,
    MACOS,
    WEBGL,
    WINDOWS,
    TVOS
}

#[derive(Debug, EnumString)]
enum TemplateStatus { NULL, DOWNLOADABLE, UPGRADABLE, READY }

#[derive(Debug)]
pub struct TemplateData {
    name: String,
    display_name: String,
    description: String,
    template_type: TemplateType,
    preview_image: String,
    render_pipeline: RenderPipeline,
    build_platforms: Vec<BuildPlatform>,
    status: TemplateStatus
}

impl TemplateData {
    fn create_template() -> TemplateData {
        return TemplateData {
            name: String::from("DEFAULT"),
            display_name: String::from("DEFAULT"),
            description: String::from("DEFAULT"),
            template_type: TemplateType::NULL,
            preview_image: String::from("DEFAULT"),
            render_pipeline: RenderPipeline::NULL,
            build_platforms: Vec::new(),
            status: TemplateStatus::NULL
        }
    }
}

fn load_json(json_str: &str) -> Result<Value, Box<dyn Error>> {
    if let json_value = serde_json::from_str(json_str)? {
        return Ok(json_value);
    }

    return Err(Box::new(error_handler::ParseFailErr("Failed to parse json.".into())));
}

fn is_tracked(path: &str) -> bool {
    match commander::run_command(String::from("ls"), vec!["-a", path]) {
        Ok((true, o)) => {
            return o.contains(".git");
        },
        _ => {
            return false;
        }
    }
}

pub fn get_project(json_value: Option<Value>, index: usize) -> Option<ProjectData> {
    let mut project: ProjectData = ProjectData::create_project();

    let loaded_value = {
        match json_value {
            Some(val) => val,
            None => {
                match commander::run_command(String::from("which"), vec!["unity"]) {
                    Ok((true, unity_path)) =>  {
                        match commander::run_command(String::from(unity_path), vec!["p", "list", "--json"]) {
                            Ok((true, o)) => {
                                match load_json(&o) {
                                    Ok(val) => val,
                                    Err(e) => {
                                        eprintln!("Failed to load json");
                                        return None;
                                    }
                                }
                            },
                            Ok((false, e)) => {
                                eprintln!("{}", e);
                                return None;
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                                return None;
                            }
                        }
                    },
                    Ok((false, e)) => {
                        eprintln!("{}", e);
                        return None;
                    },
                    Err(e) => {
                        eprintln!("{}", e);
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
                    Some(path) => project.path = String::from(path), _ => (),
                }
                match commander::run_command(String::from("ls"), vec!["-a", &project.path]) {
                    Ok((true, o)) => (),
                    _ => return None,
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

pub fn get_projects() -> Option<Vec<ProjectData>> {
    let mut projects: Vec<ProjectData> = Vec::new();

    match commander::run_command(String::from("which"), vec!["unity"]) {
        Ok((true, unity_path)) =>  {
            match commander::run_command(String::from(unity_path), vec!["p", "list", "--json"]) {
                Ok((true, o)) => {
                    match load_json(&o) {
                        Ok(json_value) => {
                            match json_value.get("data").and_then(|v| v.as_array()) {
                                Some(arr) => {
                                    let mut i = 0;
                                    while i < arr.len() {
                                        match get_project(Some(json_value.clone()), i) {
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
                Ok((false, e)) => {
                    eprintln!("{}", e);
                    return None;
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return None;
                }
            }
        },
        Ok((false, e)) => {
            eprintln!("{}", e);
            return None;
        },
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    }
}

pub fn open_project(project: &ProjectData) {
    match commander::run_command(String::from("which"), vec!["unity"]) {
        Ok((true, unity_path)) =>  {
            match commander::run_command(String::from(unity_path), vec!["p", "open", &project.path]) {
                Ok((true, o)) =>  {
                    match get_response(o) {
                        Ok((true, o)) => {
                            println!("Project opened successfully");
                        },
                        Ok((false, e)) | Err(e) => {
                            eprintln!("{}", e);
                        },
                    }
                },
                Ok((false, e)) => {
                    eprintln!("{}", e);
                },
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        },
        Ok((false, e)) => {
            eprintln!("{}", e);
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

pub fn delete_project(project: &ProjectData, remove_files: bool) {
    match commander::run_command(String::from("which"), vec!["unity"]) {
        Ok((true, unity_path)) =>  {
            match commander::run_command(String::from(unity_path), vec!["p", "remove", "-f", "--json", &project.path]) {
                Ok((true, o)) =>  {
                    match get_response(o) {
                        Ok((true, o)) => {
                            if remove_files {
                                match commander::run_command(String::from("rm"), vec!["-r", "-f", &project.path]) {
                                    Ok((true, o)) =>  {
                                        println!("project deleted successfully");
                                    },
                                    Ok((false, e)) => {
                                        eprintln!("{}", e);
                                    },
                                    Err(e) => {
                                        eprintln!("{}", e);
                                    }
                                }
                            }
                        },
                        Ok((false, e)) | Err(e) => {
                            eprintln!("{}", e);
                        }
                    }
                },
                Ok((false, e)) => {
                    eprintln!("{}", e);
                },
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
        },
        Ok((false, e)) => {
            eprintln!("{}", e);
        },
        Err(e) => {
            eprintln!("{}", e);
        }
    }
}

pub fn get_response(message: String) -> Result<(bool, String), String> {
    match load_json(&message) {
        Ok(json_value) => {
            match json_value.get("success").and_then(|v| v.as_bool()) {
                Some(s) => {
                    if s { return Ok((true, String::from(""))); }
                    else {
                        match json_value.get("data").and_then(|v| v.as_array()) {
                            Some(arr) => {
                                match json_value.get("error").and_then(|v| v.as_str()) {
                                    Some(e) => return Ok((false, String::from(e))),
                                    None => ()
                                }
                            },
                            None => ()
                        }
                    }
                },
                None => ()
            }
        },
        Err(e) => return Err(String::from("Failed to load json"))
    }
    return Err(String::from("Failed to get response"));
}

pub fn get_editors() -> Option<Vec<String>> {
    let mut editors: Vec<String> = Vec::new();

    match commander::run_command(String::from("which"), vec!["unity"]) {
        Ok((true, unity_path)) =>  {
            match commander::run_command(String::from(unity_path), vec!["editors", "list", "--installed", "--json"]) {
                Ok((true, o)) => {
                    match load_json(&o) {
                        Ok(json_value) => {
                            match json_value.get("data").and_then(|v| v.as_array()) {
                                Some(arr) => {
                                    let mut i = 0;
                                    while i < arr.len() {
                                        match arr[i].get("version").and_then(|v| v.as_str()) {
                                            Some(e) => editors.push(String::from(e)),
                                            None => ()
                                        }
                                        i += 1;
                                    }
                                    return Some(editors)
                                },
                                None => ()
                            }
                        },
                        Err(e) => ()
                    }
                },
                Ok((false, e)) => {
                    eprintln!("{}", e);
                    return None;
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return None;
                }
            }
        },
        Ok((false, e)) => {
            eprintln!("{}", e);
            return None;
        },
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    }
    None
}

pub fn get_template(json_value: Option<Value>, editor_version: Option<String>, index: usize) -> Option<TemplateData> {
    let mut template: TemplateData = TemplateData::create_template();
    let mut editor_arg = String::from("");

    match editor_version {
        Some(v) => {
            editor_arg = String::from("--editor=\"");
            editor_arg.push_str(&v);
            editor_arg.push_str("\"");
        },
        None => {
        }
    }

    let loaded_value = {
        match json_value {
            Some(val) => val,
            None => {
                match commander::run_command(String::from("which"), vec!["unity"]) {
                    Ok((true, unity_path)) =>  {
                        match commander::run_command(String::from(unity_path), vec!["templates", "list", &editor_arg, "--json"]) {
                            Ok((true, o)) => {
                                match load_json(&o) {
                                    Ok(val) => val,
                                    Err(e) => {
                                        eprintln!("Failed to load json");
                                        return None;
                                    }
                                }
                            },
                            Ok((false, e)) => {
                                eprintln!("{}", e);
                                return None;
                            },
                            Err(e) => {
                                eprintln!("{}", e);
                                return None;
                            }
                        }
                    },
                    Ok((false, e)) => {
                        eprintln!("{}", e);
                        return None;
                    },
                    Err(e) => {
                        eprintln!("{}", e);
                        return None;
                    }
                }
            }
        }
    };


    if loaded_value != Value::Null {
        match loaded_value.get("data").and_then(|v| v.as_array()) {
            Some(data) => {
                match data[index].get("name").and_then(|v| v.as_str()) {
                    Some(name) => template.name = String::from(name),
                    _ => (),
                }
                match data[index].get("displayName").and_then(|v| v.as_str()) {
                    Some(disp_name) => template.display_name = String::from(disp_name),
                    _ => (),
                }
                match data[index].get("description").and_then(|v| v.as_str()) {
                    Some(desc) => template.display_name = String::from(desc),
                    _ => (),
                }
                match data[index].get("type").and_then(|v| v.as_str()) {
                    Some(t) => template.template_type = TemplateType::from_str(t).unwrap(),
                    _ => (),
                }
                match data[index].get("previewImage").and_then(|v| v.as_str()) {
                    Some(image) => template.preview_image = String::from(image),
                    _ => (),
                }
                match data[index].get("renderPipeline").and_then(|v| v.as_str()) {
                    Some(rp) => template.render_pipeline = RenderPipeline::from_str(rp).unwrap(),
                    _ => (),
                }
                match data[index].get("buildPlatforms").and_then(|v| v.as_array()) {
                    Some(platforms) => {
                        for p in platforms {
                            template.build_platforms.push(BuildPlatform::from_str(&p.to_string()).unwrap());
                        }
                    },
                    _ => (),
                }
                match data[index].get("status").and_then(|v| v.as_str()) {
                    Some(st) => template.status = TemplateStatus::from_str(st).unwrap(),
                    _ => (),
                }
                return Some(template);
            }
            _ => (),
        }
    }
    return None;
}

pub fn get_templates(editor_version: String) -> Option<Vec<TemplateData>> {
    let mut templates: Vec<TemplateData> = Vec::new();
    let mut editor_arg = String::from("--editor=\"");
    editor_arg.push_str(&editor_version);
    editor_arg.push_str("\"");

    println!("{:?}", editor_arg);

    match commander::run_command(String::from("which"), vec!["unity"]) {
        Ok((true, o)) =>  {
            match commander::run_command(String::from(o), vec!["templates", "list", &editor_arg, "--json"]) {
                Ok((true, o)) => {
                    match load_json(&o) {
                        Ok(json_value) => {
                            match json_value.get("data").and_then(|v| v.as_array()) {
                                Some(arr) => {
                                    let mut i = 0;
                                    while i < arr.len() {
                                        match get_template(Some(json_value.clone()), None, i) {
                                            Some(t) => templates.push(t),
                                            None => ()
                                        }
                                        i += 1;
                                    }
                                    return Some(templates);
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
                Ok((false, e)) => {
                    eprintln!("{}", e);
                    return None;
                },
                Err(e) => {
                    eprintln!("{}", e);
                    return None;
                }
            }
        },
        Ok((false, e)) => {
            eprintln!("{}", e);
            return None;
        },
        Err(e) => {
            eprintln!("{}", e);
            return None;
        }
    }
}
