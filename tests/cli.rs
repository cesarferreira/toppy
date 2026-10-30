use std::process::Command;

#[test]
fn installed_command_is_tp() {
    let binary = env!("CARGO_BIN_EXE_tp");
    let output = Command::new(binary).arg("--help").output().unwrap();

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage: tp "));
}
