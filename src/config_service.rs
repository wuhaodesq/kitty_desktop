use std::fs;
use std::path::{Path, PathBuf};

use crate::KittyAdapterError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopConfig {
    pub default_directory: String,
    pub default_shell: Option<String>,
    pub default_title: String,
    pub kitty_config_file: Option<String>,
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            default_directory: ".".to_string(),
            default_shell: None,
            default_title: "kitty_desktop".to_string(),
            kitty_config_file: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigService {
    config_path: PathBuf,
}

impl ConfigService {
    pub fn new(config_path: PathBuf) -> Self {
        Self { config_path }
    }

    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    pub fn load(&self) -> Result<DesktopConfig, KittyAdapterError> {
        if !self.config_path.exists() {
            return Ok(DesktopConfig::default());
        }

        let content = fs::read_to_string(&self.config_path)?;
        parse_config_json(&content)
    }

    pub fn save(&self, config: &DesktopConfig) -> Result<(), KittyAdapterError> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = format_config_json(config);
        fs::write(&self.config_path, format!("{payload}\n"))?;
        Ok(())
    }
}

fn extract_string_field(input: &str, field: &str) -> Option<String> {
    let key = format!("\"{field}\":");
    let start = input.find(&key)? + key.len();
    let rest = input[start..].trim_start();
    if rest.starts_with("null") {
        return None;
    }
    if !rest.starts_with('"') {
        return None;
    }
    let rest = &rest[1..];
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

fn parse_config_json(input: &str) -> Result<DesktopConfig, KittyAdapterError> {
    let default_directory =
        extract_string_field(input, "default_directory").unwrap_or_else(|| ".".to_string());
    let default_title =
        extract_string_field(input, "default_title").unwrap_or_else(|| "kitty_desktop".to_string());
    let default_shell = extract_string_field(input, "default_shell");
    let kitty_config_file = extract_string_field(input, "kitty_config_file");

    Ok(DesktopConfig {
        default_directory,
        default_shell,
        default_title,
        kitty_config_file,
    })
}

fn optional_json_string(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("\"{}\"", v.replace('"', "\\\"")),
        None => "null".to_string(),
    }
}

fn format_config_json(config: &DesktopConfig) -> String {
    format!(
        "{{\n  \"default_directory\": \"{}\",\n  \"default_shell\": {},\n  \"default_title\": \"{}\",\n  \"kitty_config_file\": {}\n}}",
        config.default_directory.replace('"', "\\\""),
        optional_json_string(&config.default_shell),
        config.default_title.replace('"', "\\\""),
        optional_json_string(&config.kitty_config_file)
    )
}

pub fn render_config_json(config: &DesktopConfig) -> String {
    format_config_json(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_then_load_roundtrip() {
        let path = std::env::temp_dir().join(format!(
            "kitty_desktop_config_{}_test.json",
            std::process::id()
        ));
        let service = ConfigService::new(path.clone());

        let config = DesktopConfig {
            default_directory: "/work".to_string(),
            default_shell: Some("/bin/zsh".to_string()),
            default_title: "Dev".to_string(),
            kitty_config_file: Some("/tmp/kitty.conf".to_string()),
        };

        service.save(&config).expect("save config");
        let loaded = service.load().expect("load config");

        assert_eq!(loaded, config);
        let _ = fs::remove_file(path);
    }
}
