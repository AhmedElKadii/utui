use chrono::{DateTime, Utc};
use serde_json::Value;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct Project {
    pub git_tracked: bool,
    pub name: String,
    pub path: String,
    pub editor_version: String,
    pub template: String,
    pub last_opened: String,
}

impl Project {
    pub fn from_json(entry: &Value) -> Option<Self> {
        let path = json_str(entry, "path")?.to_string();
        if !Path::new(&path).exists() {
            return None;
        }

        Some(Self {
            name: json_str(entry, "title").unwrap_or_default().to_string(),
            git_tracked: Path::new(&path).join(".git").exists(),
            editor_version: json_str(entry, "version").unwrap_or_default().to_string(),
            last_opened: entry
                .get("lastModified")
                .and_then(Value::as_i64)
                .map(format_last_opened)
                .unwrap_or_default(),
            path,
            template: String::new(),
        })
    }

    pub fn details_text(&self) -> String {
        let git_status = if self.git_tracked {
            "tracked"
        } else {
            "untracked"
        };

        format!(
            "{name}\nLast Opened: {last_opened}\nGit Status: {git_status}\nEditor Version: {version}\nPath: {path}",
            name = self.name,
            last_opened = self.last_opened,
            version = self.editor_version,
            path = self.path
        )
    }
}

fn json_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn format_last_opened(last_modified_ms: i64) -> String {
    let secs = last_modified_ms / 1000;
    let nanos = ((last_modified_ms % 1000) as u32).saturating_mul(1_000_000);
    let Some(modified) = DateTime::from_timestamp(secs, nanos) else {
        return String::new();
    };

    let diff = Utc::now() - modified;
    if diff.num_days() == 0 {
        if diff.num_hours() < 1 {
            if diff.num_minutes() < 1 {
                format!("{} second(s) ago.", diff.num_seconds().max(0))
            } else {
                format!("{} minute(s) ago.", diff.num_minutes().max(0))
            }
        } else {
            format!("{} hour(s) ago.", diff.num_hours().max(0))
        }
    } else if diff.num_days() > 0 && diff.num_days() < 7 {
        format!("{} day(s) ago.", diff.num_days().max(0))
    } else if diff.num_days() == 7 {
        format!("{} week(s) ago.", diff.num_weeks().max(0))
    } else {
        modified.format("%d/%m/%Y").to_string()
    }
}
