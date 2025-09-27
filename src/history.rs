use std::fs::File;
use std::io::{BufRead, BufReader};

/// Read up to `n` recent history lines (newest last).
/// Uses $HISTFILE or falls back to ~/.bash_history.
pub fn import_history(n: usize) -> Vec<String> {
    let histfile = std::env::var("HISTFILE")
        .unwrap_or_else(|_| "~/.bash_history".to_string());
    let path = shellexpand::tilde(&histfile).into_owned();

    let f = match File::open(&path) {
        Ok(f) => f,
        Err(_) => return vec![],
    };

    let lines: Vec<String> = BufReader::new(f)
        .lines()
        .map_while(Result::ok)
        .collect();

    lines.into_iter().rev().take(n).rev().collect()
}
