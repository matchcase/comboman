use crate::types::Combo;
use std::process::Command;
use std::fs;
use std::env;
use tempfile::NamedTempFile;
use std::io::Write;

pub fn run_combo(combo: &Combo) {
    let script = combo.commands.join("\n");
    let mut child = Command::new("bash")
        .arg("-c")
        .arg(&script)
        .spawn()
        .expect("Failed to execute command");

    let status = child.wait().expect("failed to wait on child");
    if !status.success() {
        eprintln!("Command failed with status: {status}");
    }
}

pub fn edit_stack(stack: Vec<String>) -> Vec<String> {
    let mut file = NamedTempFile::new().unwrap();
    let script = stack.join("\n");
    file.write_all(script.as_bytes()).unwrap();
    let path = file.into_temp_path();

    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    Command::new(editor)
        .arg(&path)
        .status()
        .expect("Failed to open editor");

    let new_script = fs::read_to_string(&path).unwrap_or_default();
    new_script.lines().map(|s| s.to_string()).collect()
}
