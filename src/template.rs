use serde_json::Value;
use std::str::FromStr;
use strum::Display;
use strum_macros::EnumString;

#[derive(Debug, Clone, Default, EnumString)]
pub enum TemplateType {
    #[default]
    NULL,
    CORE,
    LEARNING,
    SAMPLE,
}

#[derive(Debug, Clone, Default, EnumString)]
pub enum RenderPipeline {
    #[default]
    NULL,
    HDRP,
    URP,
    #[allow(non_camel_case_types)]
    BUILT_IN,
}

#[derive(Debug, Clone, Default, EnumString)]
pub enum BuildPlatform {
    #[default]
    NULL,
    IOS,
    ANDROID,
    LINUX,
    MACOS,
    WEBGL,
    WINDOWS,
    TVOS,
}

#[derive(Debug, Clone, Default, EnumString, Display)]
pub enum TemplateStatus {
    #[default]
    NULL,
    DOWNLOADABLE,
    UPGRADABLE,
    READY,
}

#[derive(Debug, Clone, Default)]
pub struct Template {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub template_type: TemplateType,
    pub preview_image: String,
    pub render_pipeline: RenderPipeline,
    pub build_platforms: Vec<BuildPlatform>,
    pub status: TemplateStatus,
}

impl Template {
    pub fn from_json(entry: &Value) -> Self {
        let mut template = Self {
            name: json_str(entry, "name").unwrap_or_default().to_string(),
            display_name: json_str(entry, "displayName").unwrap_or_default().to_string(),
            description: json_str(entry, "description").unwrap_or_default().to_string(),
            preview_image: json_str(entry, "previewImage").unwrap_or_default().to_string(),
            template_type: json_str(entry, "type")
                .and_then(|value| TemplateType::from_str(value).ok())
                .unwrap_or_default(),
            render_pipeline: json_str(entry, "renderPipeline")
                .and_then(|value| RenderPipeline::from_str(value).ok())
                .unwrap_or_default(),
            status: json_str(entry, "status")
                .and_then(|value| TemplateStatus::from_str(value).ok())
                .unwrap_or_default(),
            build_platforms: Vec::new(),
        };

        if let Some(platforms) = entry.get("buildPlatforms").and_then(Value::as_array) {
            for platform in platforms {
                if let Some(name) = platform.as_str() {
                    match BuildPlatform::from_str(name) {
                        Ok(parsed) => template.build_platforms.push(parsed),
                        Err(_) => eprintln!("Unknown build platform: {name}"),
                    }
                }
            }
        }

        template
    }

    pub fn list_label(&self) -> String {
        format!(
            "{} [{}]\n{}",
            self.display_name, self.status, self.description
        )
    }
}

fn json_str<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}
