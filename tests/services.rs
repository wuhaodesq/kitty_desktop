use std::fs;

use kitty_desktop::{ConfigService, DesktopConfig, SessionService, SessionTemplate};

#[test]
fn config_service_roundtrip() {
    let config_path = std::env::temp_dir().join(format!(
        "kitty_desktop_config_integration_{}_test.json",
        std::process::id()
    ));
    let service = ConfigService::new(config_path.clone());

    let cfg = DesktopConfig {
        default_directory: "/repo".to_string(),
        default_shell: Some("/bin/bash".to_string()),
        default_title: "kitty-desktop".to_string(),
        kitty_config_file: Some("/tmp/kitty.conf".to_string()),
    };

    service.save(&cfg).expect("save config");
    let loaded = service.load().expect("load config");
    assert_eq!(loaded, cfg);

    let _ = fs::remove_file(config_path);
}

#[test]
fn session_service_upsert_and_list() {
    let session_path = std::env::temp_dir().join(format!(
        "kitty_desktop_sessions_integration_{}_test.json",
        std::process::id()
    ));
    let service = SessionService::new(session_path.clone());

    let mut session = SessionTemplate::new("dev".to_string());
    session.directory = "/repo".to_string();
    session.extra_args = vec!["--single-instance".to_string()];
    service.upsert_session(session).expect("save session");

    let sessions = service.list_sessions().expect("list sessions");
    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].name, "dev");

    let _ = fs::remove_file(session_path);
}
