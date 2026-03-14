pub mod config_service;
pub mod session_service;

use std::env;
use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KittyLaunchConfig {
    pub working_directory: Option<PathBuf>,
    pub shell: Option<String>,
    pub session_file: Option<PathBuf>,
    pub title: Option<String>,
    pub config_file: Option<PathBuf>,
    pub extra_args: Vec<String>,
}

impl KittyLaunchConfig {
    pub fn new() -> Self {
        Self {
            working_directory: None,
            shell: None,
            session_file: None,
            title: None,
            config_file: None,
            extra_args: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub enum KittyAdapterError {
    KittyNotFound,
    Io(std::io::Error),
    CommandFailed(String),
}

impl fmt::Display for KittyAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KittyNotFound => write!(
                f,
                "kitty executable was not found in PATH. Please install kitty or configure executable candidates."
            ),
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::CommandFailed(msg) => write!(f, "{msg}"),
        }
    }
}

impl Error for KittyAdapterError {}

impl From<std::io::Error> for KittyAdapterError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone)]
pub struct KittyAdapter {
    executable_candidates: Vec<String>,
}

impl Default for KittyAdapter {
    fn default() -> Self {
        Self {
            executable_candidates: vec!["kitty".to_string()],
        }
    }
}

impl KittyAdapter {
    pub fn with_candidates(candidates: Vec<String>) -> Self {
        Self {
            executable_candidates: candidates,
        }
    }

    pub fn detect_executable(&self) -> Result<PathBuf, KittyAdapterError> {
        for candidate in &self.executable_candidates {
            if let Some(found) = find_in_path(candidate) {
                return Ok(found);
            }
        }

        Err(KittyAdapterError::KittyNotFound)
    }

    pub fn build_launch_command(
        &self,
        config: &KittyLaunchConfig,
    ) -> Result<Vec<String>, KittyAdapterError> {
        let kitty_path = self.detect_executable()?;
        let mut command = vec![kitty_path.to_string_lossy().to_string()];

        if let Some(directory) = &config.working_directory {
            command.push("--directory".to_string());
            command.push(directory.to_string_lossy().to_string());
        }

        if let Some(title) = &config.title {
            command.push("--title".to_string());
            command.push(title.to_string());
        }

        if let Some(config_file) = &config.config_file {
            command.push("--config".to_string());
            command.push(config_file.to_string_lossy().to_string());
        }

        if let Some(session_file) = &config.session_file {
            command.push("--session".to_string());
            command.push(session_file.to_string_lossy().to_string());
        }

        if !config.extra_args.is_empty() {
            command.extend(config.extra_args.clone());
        }

        if let Some(shell) = &config.shell {
            command.push("--".to_string());
            command.push(shell.to_string());
        }

        Ok(command)
    }

    pub fn launch(
        &self,
        config: &KittyLaunchConfig,
        dry_run: bool,
    ) -> Result<Vec<String>, KittyAdapterError> {
        let command = self.build_launch_command(config)?;
        if dry_run {
            return Ok(command);
        }

        let child = Command::new(&command[0])
            .args(&command[1..])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let _ = child.id();

        Ok(command)
    }

    pub fn get_version(&self) -> Result<String, KittyAdapterError> {
        let kitty_path = self.detect_executable()?;
        let output = Command::new(kitty_path).arg("--version").output()?;
        if !output.status.success() {
            return Err(KittyAdapterError::CommandFailed(
                "failed to read kitty version".to_string(),
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let first_line = stdout.lines().next().unwrap_or_default().trim().to_string();
        if first_line.is_empty() {
            return Err(KittyAdapterError::CommandFailed(
                "kitty returned empty version output".to_string(),
            ));
        }

        Ok(first_line)
    }
}

fn find_in_path(executable: &str) -> Option<PathBuf> {
    let candidate_path = Path::new(executable);
    if candidate_path.components().count() > 1 && candidate_path.exists() {
        return Some(candidate_path.to_path_buf());
    }

    let path_var = env::var_os("PATH")?;
    for path_dir in env::split_paths(&path_var) {
        let full_path = path_dir.join(executable);
        if full_path.is_file() {
            return Some(full_path);
        }
    }

    None
}

pub fn normalize_extra_args(extra_args: &[String]) -> Vec<String> {
    if !extra_args.is_empty() && extra_args[0] == "--" {
        return extra_args[1..].to_vec();
    }

    extra_args.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_extra_args_strips_separator() {
        let source = vec!["--".to_string(), "--single-instance".to_string()];
        assert_eq!(normalize_extra_args(&source), vec!["--single-instance"]);
    }

    #[test]
    fn normalize_extra_args_without_separator() {
        let source = vec!["--single-instance".to_string()];
        assert_eq!(normalize_extra_args(&source), source);
    }

    #[test]
    fn build_launch_command_with_options() {
        let mut fake_kitty = std::env::temp_dir();
        fake_kitty.push("kitty_desktop_fake_kitty");
        std::fs::write(&fake_kitty, "#!/bin/sh\n").expect("write fake kitty");

        let adapter = KittyAdapter::with_candidates(vec![fake_kitty.to_string_lossy().to_string()]);
        let config = KittyLaunchConfig {
            working_directory: Some(PathBuf::from("/work")),
            shell: Some("/bin/zsh".to_string()),
            session_file: Some(PathBuf::from("/tmp/session.conf")),
            title: Some("Dev".to_string()),
            config_file: Some(PathBuf::from("/tmp/kitty.conf")),
            extra_args: vec!["--single-instance".to_string()],
        };

        let command = adapter.build_launch_command(&config);
        if cfg!(target_os = "windows") {
            assert!(command.is_err());
        } else {
            let expected = vec![
                fake_kitty.to_string_lossy().to_string(),
                "--directory".to_string(),
                "/work".to_string(),
                "--title".to_string(),
                "Dev".to_string(),
                "--config".to_string(),
                "/tmp/kitty.conf".to_string(),
                "--session".to_string(),
                "/tmp/session.conf".to_string(),
                "--single-instance".to_string(),
                "--".to_string(),
                "/bin/zsh".to_string(),
            ];
            assert_eq!(command.unwrap(), expected);
        }
    }
}

pub use config_service::{ConfigService, DesktopConfig};
pub use session_service::{SessionService, SessionTemplate};
