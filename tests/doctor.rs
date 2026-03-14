use std::process::Command;

#[test]
fn doctor_command_prints_checks() {
    let bin = env!("CARGO_BIN_EXE_kitty_desktop");
    let output = Command::new(bin)
        .arg("doctor")
        .output()
        .expect("run doctor");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("kitty_executable"));
    assert!(stdout.contains("config_load"));
    assert!(stdout.contains("session_list"));
}
