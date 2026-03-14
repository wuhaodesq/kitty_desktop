use std::io::{self, BufRead, Write};
use std::path::PathBuf;

use crate::{
    ConfigService, DesktopConfig, KittyAdapter, KittyAdapterError, KittyLaunchConfig,
    SessionService,
};

#[derive(Debug, Clone)]
pub struct DesktopShell {
    pub adapter: KittyAdapter,
    pub config_service: ConfigService,
    pub session_service: SessionService,
}

impl DesktopShell {
    pub fn new(config_path: PathBuf, session_path: PathBuf) -> Self {
        Self::new_with_adapter(KittyAdapter::default(), config_path, session_path)
    }

    pub fn new_with_adapter(
        adapter: KittyAdapter,
        config_path: PathBuf,
        session_path: PathBuf,
    ) -> Self {
        Self {
            adapter,
            config_service: ConfigService::new(config_path),
            session_service: SessionService::new(session_path),
        }
    }

    pub fn launch_from_profile(
        &self,
        profile_name: Option<&str>,
        dry_run: bool,
    ) -> Result<Vec<String>, KittyAdapterError> {
        let config = self.config_service.load()?;
        let mut launch = KittyLaunchConfig::new();

        launch.working_directory = Some(PathBuf::from(config.default_directory));
        launch.shell = config.default_shell;
        launch.title = Some(config.default_title);
        launch.config_file = config.kitty_config_file.map(PathBuf::from);

        if let Some(name) = profile_name {
            if let Some(session) = self.session_service.get_session(name)? {
                launch.working_directory = Some(PathBuf::from(session.directory));
                launch.shell = session.shell;
                launch.title = session.title;
                launch.extra_args = session.extra_args;
            } else {
                return Err(KittyAdapterError::CommandFailed(format!(
                    "session profile not found: {name}"
                )));
            }
        }

        self.adapter.launch(&launch, dry_run)
    }

    pub fn save_settings(
        &self,
        directory: Option<String>,
        shell: Option<String>,
        title: Option<String>,
        kitty_config: Option<String>,
    ) -> Result<DesktopConfig, KittyAdapterError> {
        let mut cfg = self.config_service.load()?;
        if let Some(v) = directory {
            cfg.default_directory = v;
        }
        if let Some(v) = shell {
            cfg.default_shell = Some(v);
        }
        if let Some(v) = title {
            cfg.default_title = v;
        }
        if let Some(v) = kitty_config {
            cfg.kitty_config_file = Some(v);
        }

        self.config_service.save(&cfg)?;
        Ok(cfg)
    }

    pub fn run_repl<R: BufRead, W: Write>(
        &self,
        reader: &mut R,
        writer: &mut W,
    ) -> Result<(), KittyAdapterError> {
        writeln!(
            writer,
            "kitty_desktop shell (commands: launch [name], settings, exit)"
        )
        .map_err(KittyAdapterError::Io)?;

        loop {
            write!(writer, "> ").map_err(KittyAdapterError::Io)?;
            writer.flush().map_err(KittyAdapterError::Io)?;

            let mut line = String::new();
            let bytes = reader.read_line(&mut line).map_err(KittyAdapterError::Io)?;
            if bytes == 0 {
                break;
            }

            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            if line == "exit" || line == "quit" {
                writeln!(writer, "bye").map_err(KittyAdapterError::Io)?;
                break;
            }

            if line == "settings" {
                let cfg = self.config_service.load()?;
                writeln!(writer, "directory={}", cfg.default_directory)
                    .map_err(KittyAdapterError::Io)?;
                writeln!(writer, "shell={}", cfg.default_shell.unwrap_or_default())
                    .map_err(KittyAdapterError::Io)?;
                writeln!(writer, "title={}", cfg.default_title).map_err(KittyAdapterError::Io)?;
                continue;
            }

            if let Some(rest) = line.strip_prefix("launch") {
                let profile = rest.trim();
                let profile = if profile.is_empty() {
                    None
                } else {
                    Some(profile)
                };
                match self.launch_from_profile(profile, true) {
                    Ok(cmd) => {
                        writeln!(writer, "{}", cmd.join(" ")).map_err(KittyAdapterError::Io)?;
                    }
                    Err(err) => {
                        writeln!(writer, "ERROR: {err}").map_err(KittyAdapterError::Io)?;
                    }
                }
                continue;
            }

            writeln!(writer, "unknown command: {line}").map_err(KittyAdapterError::Io)?;
        }

        Ok(())
    }

    pub fn run_with_stdio(&self) -> Result<(), KittyAdapterError> {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        let mut stdout = io::stdout();
        self.run_repl(&mut reader, &mut stdout)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SessionService, SessionTemplate};
    use std::fs;
    use std::io::Cursor;

    fn paths() -> (PathBuf, PathBuf) {
        let id = unique_id();
        let base =
            std::env::temp_dir().join(format!("kitty_desktop_shell_{}_{}", std::process::id(), id));
        (base.join("config.json"), base.join("sessions.json"))
    }

    fn unique_id() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    }

    #[test]
    fn launch_from_default_profile_uses_config() {
        let (config_path, session_path) = paths();
        let fake = std::env::temp_dir().join(format!("kitty_shell_fake_kitty_{}", unique_id()));
        let _ = std::fs::write(&fake, "#!/bin/sh\n");
        let adapter = KittyAdapter::with_candidates(vec![fake.to_string_lossy().to_string()]);
        let shell =
            DesktopShell::new_with_adapter(adapter, config_path.clone(), session_path.clone());

        shell
            .config_service
            .save(&DesktopConfig {
                default_directory: "/tmp".to_string(),
                default_shell: Some("/bin/bash".to_string()),
                default_title: "dev".to_string(),
                kitty_config_file: None,
            })
            .expect("save");

        let out = shell.launch_from_profile(None, true).expect("dry run");
        assert_eq!(out[0], fake.to_string_lossy());

        let _ = fs::remove_file(&fake);
        let _ = fs::remove_file(config_path);
        let _ = fs::remove_file(session_path);
    }

    #[test]
    fn repl_exit_works() {
        let (config_path, session_path) = paths();
        let shell = DesktopShell::new(config_path, session_path);

        let input = b"exit\n";
        let mut reader = Cursor::new(&input[..]);
        let mut output = Vec::new();
        shell.run_repl(&mut reader, &mut output).expect("repl");

        let text = String::from_utf8(output).expect("utf8");
        assert!(text.contains("kitty_desktop shell"));
        assert!(text.contains("bye"));
    }

    #[test]
    fn save_settings_updates_config() {
        let (config_path, session_path) = paths();
        let shell = DesktopShell::new(config_path.clone(), session_path);

        let saved = shell
            .save_settings(
                Some("/work".to_string()),
                Some("/bin/zsh".to_string()),
                Some("Dev".to_string()),
                None,
            )
            .expect("save settings");

        assert_eq!(saved.default_directory, "/work");

        let loaded = shell.config_service.load().expect("load");
        assert_eq!(loaded.default_title, "Dev");
        let _ = fs::remove_file(config_path);
    }

    #[test]
    fn launch_with_session_profile_name_not_found() {
        let (config_path, session_path) = paths();
        let fake = std::env::temp_dir().join(format!("kitty_shell_fake_kitty_{}", unique_id()));
        let _ = std::fs::write(&fake, "#!/bin/sh\n");
        let adapter = KittyAdapter::with_candidates(vec![fake.to_string_lossy().to_string()]);
        let shell =
            DesktopShell::new_with_adapter(adapter, config_path.clone(), session_path.clone());
        shell
            .config_service
            .save(&DesktopConfig::default())
            .expect("save");
        let err = shell
            .launch_from_profile(Some("missing"), true)
            .expect_err("err");
        assert!(format!("{err}").contains("session profile not found"));
        let _ = fs::remove_file(&fake);
        let _ = fs::remove_file(config_path);
        let _ = fs::remove_file(session_path);
    }

    #[test]
    fn launch_with_session_profile_uses_session_data() {
        let (config_path, session_path) = paths();
        let fake = std::env::temp_dir().join(format!("kitty_shell_fake_kitty_{}", unique_id()));
        let _ = std::fs::write(&fake, "#!/bin/sh\n");
        let adapter = KittyAdapter::with_candidates(vec![fake.to_string_lossy().to_string()]);
        let shell =
            DesktopShell::new_with_adapter(adapter, config_path.clone(), session_path.clone());
        shell
            .config_service
            .save(&DesktopConfig::default())
            .expect("save");
        let mut s = SessionTemplate::new("dev".to_string());
        s.directory = "/repo".to_string();
        s.extra_args = vec!["--single-instance".to_string()];
        SessionService::new(session_path.clone())
            .upsert_session(s)
            .expect("upsert");

        let out = shell
            .launch_from_profile(Some("dev"), true)
            .expect("dry run");
        assert_eq!(out[0], fake.to_string_lossy());
        assert!(out.iter().any(|x| x == "--single-instance"));

        let _ = fs::remove_file(&fake);
        let _ = fs::remove_file(config_path);
        let _ = fs::remove_file(session_path);
    }
}
