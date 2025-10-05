use std::fs::File;
use std::io::{BufRead, BufReader};

/// Read up to `n` recent history lines (newest last).
/// Uses $HISTFILE or falls back to shell-specific defaults.
pub fn import_history(n: usize) -> Vec<String> {
    let shell = std::env::var("SHELL").unwrap_or_default();
    let histfile = if shell.ends_with("zsh") {
        std::env::var("HISTFILE").unwrap_or_else(|_| "~/.zsh_history".to_string())
    } else if shell.ends_with("fish") {
        "~/.local/share/fish/fish_history".to_string()
    } else {
        std::env::var("HISTFILE").unwrap_or_else(|_| "~/.bash_history".to_string())
    };

    let path = shellexpand::tilde(&histfile).into_owned();

    let f = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let lines: Vec<String> = BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .collect();

    if shell.ends_with("fish") {
        let fish_cmds: Vec<String> = lines
            .into_iter()
            .filter_map(|line| line.strip_prefix("- cmd: ").map(|s| s.to_string()))
            .collect();
        fish_cmds.into_iter().rev().take(n).rev().collect()
    } else {
        lines.into_iter().rev().take(n).rev().collect()
    }
}
