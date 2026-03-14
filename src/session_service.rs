use std::fs;
use std::path::PathBuf;

use crate::KittyAdapterError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionTemplate {
    pub name: String,
    pub directory: String,
    pub shell: Option<String>,
    pub title: Option<String>,
    pub extra_args: Vec<String>,
}

impl SessionTemplate {
    pub fn new(name: String) -> Self {
        Self {
            name,
            directory: ".".to_string(),
            shell: None,
            title: None,
            extra_args: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionService {
    session_store_path: PathBuf,
}

impl SessionService {
    pub fn new(session_store_path: PathBuf) -> Self {
        Self { session_store_path }
    }

    pub fn store_path(&self) -> &PathBuf {
        &self.session_store_path
    }

    pub fn list_sessions(&self) -> Result<Vec<SessionTemplate>, KittyAdapterError> {
        if !self.session_store_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.session_store_path)?;
        parse_sessions_json(&content)
    }

    pub fn upsert_session(&self, template: SessionTemplate) -> Result<(), KittyAdapterError> {
        let mut sessions = self.list_sessions()?;

        if let Some(existing) = sessions.iter_mut().find(|item| item.name == template.name) {
            *existing = template;
        } else {
            sessions.push(template);
        }

        self.save(&sessions)
    }

    pub fn get_session(&self, name: &str) -> Result<Option<SessionTemplate>, KittyAdapterError> {
        let sessions = self.list_sessions()?;
        Ok(sessions.into_iter().find(|item| item.name == name))
    }

    fn save(&self, sessions: &[SessionTemplate]) -> Result<(), KittyAdapterError> {
        if let Some(parent) = self.session_store_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let payload = format_sessions_json(sessions);
        fs::write(&self.session_store_path, format!("{payload}\n"))?;
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

fn extract_extra_args(input: &str) -> Vec<String> {
    let key = "\"extra_args\":";
    let Some(start) = input.find(key) else {
        return Vec::new();
    };
    let rest = input[start + key.len()..].trim_start();
    if !rest.starts_with('[') {
        return Vec::new();
    }
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };
    let inner = &rest[1..end];
    inner
        .split(',')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.trim_matches('"').to_string())
        .collect()
}

fn parse_session_object(input: &str) -> Option<SessionTemplate> {
    let name = extract_string_field(input, "name")?;
    let directory = extract_string_field(input, "directory").unwrap_or_else(|| ".".to_string());
    let shell = extract_string_field(input, "shell");
    let title = extract_string_field(input, "title");
    let extra_args = extract_extra_args(input);

    Some(SessionTemplate {
        name,
        directory,
        shell,
        title,
        extra_args,
    })
}

fn parse_sessions_json(input: &str) -> Result<Vec<SessionTemplate>, KittyAdapterError> {
    let mut sessions = Vec::new();
    let mut rest = input;

    while let Some(start) = rest.find('{') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find('}') else {
            break;
        };
        let object = &after_start[..end];
        if let Some(session) = parse_session_object(object) {
            sessions.push(session);
        }
        rest = &after_start[end + 1..];
    }

    Ok(sessions)
}

fn optional_json_string(value: &Option<String>) -> String {
    match value {
        Some(v) => format!("\"{}\"", v.replace('"', "\\\"")),
        None => "null".to_string(),
    }
}

fn format_extra_args(extra_args: &[String]) -> String {
    let values = extra_args
        .iter()
        .map(|arg| format!("\"{}\"", arg.replace('"', "\\\"")))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{values}]")
}

fn format_sessions_json(sessions: &[SessionTemplate]) -> String {
    let body = sessions
        .iter()
        .map(|session| {
            format!(
                "  {{\n    \"name\": \"{}\",\n    \"directory\": \"{}\",\n    \"shell\": {},\n    \"title\": {},\n    \"extra_args\": {}\n  }}",
                session.name.replace('"', "\\\""),
                session.directory.replace('"', "\\\""),
                optional_json_string(&session.shell),
                optional_json_string(&session.title),
                format_extra_args(&session.extra_args)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");

    format!("[\n{body}\n]")
}

pub fn render_sessions_json(sessions: &[SessionTemplate]) -> String {
    format_sessions_json(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_and_get_session() {
        let path = std::env::temp_dir().join(format!(
            "kitty_desktop_sessions_{}_test.json",
            std::process::id()
        ));
        let service = SessionService::new(path.clone());

        let mut first = SessionTemplate::new("dev".to_string());
        first.directory = "/work".to_string();
        first.title = Some("Dev".to_string());

        service.upsert_session(first).expect("save first");

        let mut replacement = SessionTemplate::new("dev".to_string());
        replacement.directory = "/srv".to_string();
        service.upsert_session(replacement).expect("update");

        let loaded = service
            .get_session("dev")
            .expect("get")
            .expect("session exists");

        assert_eq!(loaded.directory, "/srv");
        let _ = fs::remove_file(path);
    }
}
