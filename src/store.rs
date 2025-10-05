use crate::types::Combo;
use std::fs::{create_dir_all, File, OpenOptions};
use std::path::{Path, PathBuf};
use chrono::Utc;

/// Returns the path to the combo file, ensuring the directory exists.
fn get_combo_file_path(combo_directory: Option<String>) -> PathBuf {
    let mut path = match combo_directory {
        Some(dir) => PathBuf::from(dir),
        None => {
            let config_dir = dirs::config_dir().expect("Cannot find config directory");
            let mut path = config_dir;
            path.push("comboman");
            path
        }
    };
    create_dir_all(&path).expect("Cannot create comboman directory");
    path.push("combos.yaml");
    path
}

pub fn load_combos(combo_directory: Option<String>) -> Vec<Combo> {
    let path = get_combo_file_path(combo_directory);
    if !Path::new(&path).exists() {
        return vec![];
    }
    let f = File::open(path).expect("Cannot open combo file");
    serde_yaml::from_reader(f).unwrap_or_else(|_| vec![])
}

pub fn save_combos(combos: &[Combo], combo_directory: Option<String>) {
    let path = get_combo_file_path(combo_directory);
    let f = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .expect("Cannot write combo file");
    serde_yaml::to_writer(f, combos).expect("Failed to serialize combos");
}

pub fn update_last_used(combos: &mut [Combo], name: &str) {
    if let Some(c) = combos.iter_mut().find(|c| c.name == name) {
        c.last_used = Utc::now().timestamp();
    }
}

/// Add a combo with optional name. If name is None, generate fallback
/// using sanitize_name(first_command) + _i to avoid collisions.
pub fn add_combo(combos: &mut Vec<Combo>, commands: Vec<String>, name: Option<String>, combo_directory: Option<String>) {
    let now = Utc::now();
    let combo_name = name.unwrap_or_else(|| {
        let base = commands
            .last()
            .map(|s| sanitize_name(s))
            .unwrap_or_else(|| "combo".to_string());
        let mut i = 0;
        loop {
            let candidate = format!("{base}_{i}");
            if !combos.iter().any(|c| c.name == candidate) {
                break candidate;
            }
            i += 1;
        }
    });

    combos.push(Combo {
        name: combo_name,
        commands,
        last_used: now.timestamp(),
    });

    save_combos(combos, combo_directory);
}

/// Sanitize a command token into a safe base name.
fn sanitize_name(cmd: &str) -> String {
    let first_token = cmd.split_whitespace().next().unwrap_or("combo");
    first_token
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>()
        .to_lowercase()
        .chars()
        .take(64)
        .collect()
}
