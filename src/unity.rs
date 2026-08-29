use serde_json::Value;
use crate::command::run_command;
use crate::error::AppError;
use crate::project::Project;
use crate::template::Template;

pub struct UnityCLI {
    binary: String,
}

impl UnityCLI {
    pub fn discover() -> Result<Self, AppError> {
        match run_command("which", &["unity"]) {
            Ok((true, path)) => Ok(Self {
                binary: path.trim().to_string(),
            }),
            Ok((false, err)) => {
                Err(AppError::Unity(err))
            }
            Err(err) => Err(err.into()),
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

    pub fn list_offline_templates(&self, editor_version: &str) -> Result<Vec<Template>, AppError> {
        let editor_arg = format!("--editor={editor_version}");
        let json = self.invoke(&["templates", "list", &editor_arg, "--installed", "--json"])?;
        let data = json_data_array(&json)?;
        Ok(data.iter().map(Template::from_json).collect())
    }

    pub fn open_project(&self, project: &Project) -> Result<(), AppError> {
        let output_result = self.raw(&["p", "open", &project.path]);

        let message_text = match output_result {
            Ok(stdout_string) => stdout_string,
            Err(app_error) => {
                let err_string = app_error.to_string();
                if err_string.is_empty() {
                    "".to_string()
                } else {
                    err_string
                }
            }
        };

        parse_cli_response(&message_text)?;

        Ok(())
    }

    pub fn delete_project(&self, project: &Project, remove_files: bool) -> Result<(), AppError> {
        let output_result = self.raw(&["p", "remove", "-f", &project.path, "--json"]);

        let message_text = match output_result {
            Ok(stdout_string) => stdout_string,
            Err(app_error) => {
                let err_string = app_error.to_string();
                if err_string.is_empty() {
                    "".to_string()
                } else {
                    err_string
                }
            }
        };

        parse_cli_response(&message_text)?;

        if remove_files {
            if let Err(err) = run_command("rm", &["-r", "-f", &project.path]) {
                return Err(AppError::Command(err));
            }
        }

        Ok(())
    }

    pub fn create_project(&self, project: &Project) -> Result<(), AppError> {
        let editor_arg = format!("--editor-version={}", project.editor_version);
        let template_arg = format!("--template={}", project.template);
        let path_arg = format!("--path={}", project.path);

        let output = self.raw(&[
            "p",
            "create",
            &project.name,
            &editor_arg,
            &template_arg,
            &path_arg,
            "--json",
        ])?;
        
        parse_cli_response(&output)?;
        Ok(())
    }

    fn invoke(&self, args: &[&str]) -> Result<Value, AppError> {
        let stdout = self.raw(args)?;
        serde_json::from_str(&stdout).map_err(AppError::from)
    }

    fn raw(&self, args: &[&str]) -> Result<String, AppError> {
        match run_command(&self.binary, args) {
            Ok((true, stdout)) => Ok(stdout),
            Ok((false, err)) => Err(AppError::Unity(err)),
            Err(err) => Err(err.into()),
        }
    }
}

fn json_data_array(value: &Value) -> Result<&Vec<Value>, AppError> {
    match value.get("data").and_then(Value::as_array) {
        Some(data) => Ok(data),
        None => Err(AppError::Parse("Failed to fetch json array".into())),
    }
}

fn parse_cli_response(message: &str) -> Result<(), AppError> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    let json: Value = match serde_json::from_str(trimmed) {
        Ok(val) => val,
        Err(_) => return Ok(()), 
    };

    match json.get("success").and_then(Value::as_bool) {
        Some(false) => {
            if let Some(err_str) = json.get("error").and_then(Value::as_str) {
                if !err_str.trim().is_empty() {
                    return Err(AppError::Parse(err_str.to_string()));
                }
            }
            Ok(())
        }
        _ => Ok(()), 
    }
}
