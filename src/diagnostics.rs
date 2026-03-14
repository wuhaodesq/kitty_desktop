use std::path::Path;

use crate::{ConfigService, KittyAdapter, KittyAdapterError, SessionService};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckResult {
    pub name: &'static str,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticsReport {
    pub checks: Vec<CheckResult>,
}

impl DiagnosticsReport {
    pub fn is_healthy(&self) -> bool {
        self.checks.iter().all(|item| item.ok)
    }

    pub fn render_text(&self) -> String {
        self.checks
            .iter()
            .map(|item| {
                let status = if item.ok { "OK" } else { "FAIL" };
                format!("[{status}] {}: {}", item.name, item.detail)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

pub fn run_diagnostics(
    adapter: &KittyAdapter,
    config_service: &ConfigService,
    session_service: &SessionService,
) -> Result<DiagnosticsReport, KittyAdapterError> {
    let mut checks = Vec::new();

    checks.push(match adapter.detect_executable() {
        Ok(path) => CheckResult {
            name: "kitty_executable",
            ok: true,
            detail: format!("found at {}", path.display()),
        },
        Err(_) => CheckResult {
            name: "kitty_executable",
            ok: false,
            detail: "kitty not found in PATH".to_string(),
        },
    });

    checks.push(match adapter.get_version() {
        Ok(version) => CheckResult {
            name: "kitty_version",
            ok: true,
            detail: version,
        },
        Err(err) => CheckResult {
            name: "kitty_version",
            ok: false,
            detail: err.to_string(),
        },
    });

    checks.push(check_parent_dir(
        "config_parent_dir",
        config_service.config_path(),
    ));

    let config_load_ok = config_service.load().is_ok();
    checks.push(CheckResult {
        name: "config_load",
        ok: config_load_ok,
        detail: if config_load_ok {
            "config load OK".to_string()
        } else {
            "config load failed".to_string()
        },
    });

    checks.push(check_parent_dir(
        "session_parent_dir",
        session_service.store_path(),
    ));

    let session_list_ok = session_service.list_sessions().is_ok();
    checks.push(CheckResult {
        name: "session_list",
        ok: session_list_ok,
        detail: if session_list_ok {
            "session list OK".to_string()
        } else {
            "session list failed".to_string()
        },
    });

    Ok(DiagnosticsReport { checks })
}

fn check_parent_dir(name: &'static str, target_path: &Path) -> CheckResult {
    let parent = target_path.parent();
    match parent {
        Some(dir) => {
            if dir.exists() {
                CheckResult {
                    name,
                    ok: true,
                    detail: format!("{} exists", dir.display()),
                }
            } else {
                CheckResult {
                    name,
                    ok: true,
                    detail: format!("{} will be created on first write", dir.display()),
                }
            }
        }
        None => CheckResult {
            name,
            ok: false,
            detail: "target has no parent directory".to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConfigService, SessionService};
    use std::path::PathBuf;

    #[test]
    fn report_health_reflects_failed_checks() {
        let report = DiagnosticsReport {
            checks: vec![
                CheckResult {
                    name: "a",
                    ok: true,
                    detail: "ok".to_string(),
                },
                CheckResult {
                    name: "b",
                    ok: false,
                    detail: "bad".to_string(),
                },
            ],
        };

        assert!(!report.is_healthy());
        assert!(report.render_text().contains("[FAIL] b"));
    }

    #[test]
    fn diagnostics_run_produces_expected_checks() {
        let adapter = KittyAdapter::default();
        let config_service = ConfigService::new(PathBuf::from(".kitty_desktop/config.json"));
        let session_service = SessionService::new(PathBuf::from(".kitty_desktop/sessions.json"));

        let report = run_diagnostics(&adapter, &config_service, &session_service).expect("report");
        assert!(report.checks.iter().any(|x| x.name == "kitty_executable"));
        assert!(report.checks.iter().any(|x| x.name == "config_load"));
        assert!(report.checks.iter().any(|x| x.name == "session_list"));
    }
}
