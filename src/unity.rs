use serde_json::Value;

use crate::command::run_command;
use crate::error::AppError;
use crate::project::Project;
use crate::template::Template;

pub struct UnityHub {
    binary: String,
}

impl UnityHub {
    pub fn discover() -> Result<Self, AppError> {
        match run_command("which", &["unity"]) {
            Ok((true, path)) => Ok(Self {
                binary: path.trim().to_string(),
            }),
            Ok((false, err)) => {
                eprintln!("{err}");
                Err(AppError::Unity(err))
            }
            Err(err) => {
                eprintln!("{err}");
                Err(err.into())
            }
        }
    }

    pub fn list_projects(&self) -> Result<Vec<Project>, AppError> {
        let json = self.invoke(&["p", "list", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().filter_map(Project::from_json).collect())
    }

    pub fn list_editors(&self) -> Result<Vec<String>, AppError> {
        let json = self.invoke(&["editors", "list", "--installed", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data
            .iter()
            .filter_map(|entry| {
                entry
                    .get("version")
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned)
            })
            .collect())
    }

    pub fn list_templates(&self, editor_version: &str) -> Result<Vec<Template>, AppError> {
        let editor_arg = format!("--editor={editor_version}");
        let json = self.invoke(&["templates", "list", &editor_arg, "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().map(Template::from_json).collect())
    }

    pub fn open_project(&self, project: &Project) -> Result<(), AppError> {
        let output = self.raw(&["p", "open", &project.path])?;
        match parse_cli_response(&output) {
            Ok(true) => println!("Project opened successfully"),
            Ok(false) | Err(_) => {}
        }
        Ok(())
    }

    pub fn delete_project(&self, project: &Project, remove_files: bool) -> Result<(), AppError> {
        let output = self.raw(&["p", "remove", "-f", "--json", &project.path])?;
        if !matches!(parse_cli_response(&output), Ok(true)) {
            return Ok(());
        }

        if remove_files {
            match run_command("rm", &["-r", "-f", &project.path]) {
                Ok((true, _)) => println!("project deleted successfully"),
                Ok((false, err)) => eprintln!("{err}"),
                Err(err) => eprintln!("{err}"),
            }
        }

        Ok(())
    }

    pub fn create_project(&self, project: &Project) -> Result<(), AppError> {
        let editor_arg = format!("--editor-version={}", project.editor_version);
        let template_arg = format!("--template={}", project.template);
        let path_arg = format!("--path={}", project.path);

        match self.raw(&[
            "p",
            "create",
            &project.name,
            &editor_arg,
            &template_arg,
            &path_arg,
            "--json",
        ]) {
            Ok(_) => {
                println!("Project created successfully");
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn invoke(&self, args: &[&str]) -> Result<Value, AppError> {
        let stdout = self.raw(args)?;
        serde_json::from_str(&stdout).map_err(AppError::from)
    }

    fn raw(&self, args: &[&str]) -> Result<String, AppError> {
        match run_command(&self.binary, args) {
            Ok((true, stdout)) => Ok(stdout),
            Ok((false, err)) => {
                eprintln!("{err}");
                Err(AppError::Unity(err))
            }
            Err(err) => {
                eprintln!("{err}");
                Err(err.into())
            }
        }
    }
}

fn json_data_array(value: &Value) -> Result<&Vec<Value>, AppError> {
    match value.get("data").and_then(Value::as_array) {
        Some(data) => Ok(data),
        None => {
            eprintln!("Failed to fetch json array");
            Err(AppError::Parse("Failed to fetch json array".into()))
        }
    }
}

fn parse_cli_response(message: &str) -> Result<bool, AppError> {
    let json: Value = serde_json::from_str(message).map_err(|_| {
        AppError::Parse("Failed to load json".into())
    })?;

    match json.get("success").and_then(Value::as_bool) {
        Some(true) => Ok(true),
        Some(false) => {
            if json.get("data").and_then(Value::as_array).is_some() {
                if let Some(err) = json.get("error").and_then(Value::as_str) {
                    eprintln!("{err}");
                    return Ok(false);
                }
            }
            Err(AppError::Parse("Failed to get response".into()))
        }
        None => Err(AppError::Parse("Failed to get response".into())),
    }
}
