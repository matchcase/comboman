mod types;
mod store;
mod ui;
mod exec;
mod history;

use std::env;
use std::io::{stdin, stdout, Write};

use crate::exec::{edit_stack, run_combo};
use crate::history::import_history;
use crate::store::{add_combo, load_combos, save_combos, update_last_used};
use crate::ui::{run_ui, select_save_option, select_stack};
use crate::types::{Combo, SaveOption};
use crossterm::{
    cursor::MoveTo,
    execute,
    terminal::{Clear, ClearType},
};
use std::fs;
use std::path::Path;

fn list_combos(combos: &[Combo]) {
    let mut sorted = combos.to_vec();
    sorted.sort_by(|a, b| b.last_used.cmp(&a.last_used));
    for c in sorted {
        println!("{} (last used: {})", c.name, c.last_used);
    }
}

fn delete_combo(combos: &mut Vec<Combo>, name: &str) {
    let before = combos.len();
    combos.retain(|c| c.name != name);
    if combos.len() < before {
        save_combos(combos);
        println!("Deleted combo '{name}'");
    } else {
        eprintln!("Combo '{name}' not found");
    }
}

fn prompt_line(prompt: &str) -> Option<String> {
    print!("{prompt}");
    stdout().flush().ok()?;
    let mut line = String::new();
    stdin().read_line(&mut line).ok()?;
    let s = line.trim();
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

fn main() {
    let mut combos = load_combos();
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("list") => list_combos(&combos),
        Some("delete") => {
            if let Some(name) = args.get(2) {
                delete_combo(&mut combos, name);
            } else {
                eprintln!("Usage: comboman delete <name>");
            }
        }
        Some("new") => {
            // Import recent history and open stack-like selector
            let recent_cmds = import_history(200); // show up to 200 recent commands
            if recent_cmds.is_empty() {
                println!("No history found.");
                return;
            }
            let mut stack = match select_stack(recent_cmds, None) {
                Some(s) if !s.is_empty() => s,
                _ => {
                    println!("No commands selected or selection cancelled.");
                    return;
                }
            };

            loop {
                match select_save_option(&stack) {
                    Some(SaveOption::Edit) => {
                        stack = edit_stack(stack);
                        execute!(stdout(), Clear(ClearType::All), MoveTo(0, 0)).unwrap();
                    }
                    Some(SaveOption::SaveAsScript) => {
                        if let Some(path) = prompt_line("Enter path to save script: ") {
                            let script = stack.join("\n");
                            if fs::write(&path, script).is_ok() {
                                println!("Saved script to {path}");
                            } else {
                                eprintln!("Failed to save script to {path}");
                            }
                        }
                        break;
                    }
                    Some(SaveOption::SaveAsFunction) => {
                        let name = prompt_line("Enter function name (leave blank for default): ");
                        let func_name =
                            name.unwrap_or_else(|| format!("command_{}", combos.len() + 1));
                        let function =
                            format!("\n{} () {{ \n{}\n }}\n", func_name, stack.join("\n"));

                        // Simplified: Append to .bashrc. A real implementation would need to be more robust.
                        let shell_rc = Path::new(&env::var("HOME").unwrap()).join(".bashrc");
                        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&shell_rc) {
                            if file.write_all(function.as_bytes()).is_ok() {
                                println!("Added function '{func_name}' to your shell rc file.");
                                println!("Run 'source {}' to use it.", shell_rc.display());
                            } else {
                                eprintln!("Failed to write to shell rc file.");
                            }
                        } else {
                            eprintln!("Could not open shell rc file.");
                        }
                        break;
                    }
                    Some(SaveOption::SaveAsCombo) => {
                        let name =
                            prompt_line("Enter name for combo (leave blank to auto-generate): ");
                        add_combo(&mut combos, stack, name);
                        println!("Combo saved.");
                        break;
                    }
                    None => {
                        println!("Selection cancelled.");
                        break;
                    }
                }
            }
        }
        Some("run") | None => {
            // Interactive run UI
            if combos.is_empty() {
                println!("No saved combos. Use 'comboman new' to create one.");
                return;
            }
            if let Some(name) = run_ui(combos.clone()) {
                if let Some(combo) = combos.iter().find(|c| c.name == name) {
                    run_combo(combo);
                    update_last_used(&mut combos, &name);
                    save_combos(&combos);
                } else {
                    eprintln!("Selected combo '{name}' not found (concurrent modification?).");
                }
            } else {
                println!("Cancelled.");
            }
        }
        Some(cmd) => {
            eprintln!("Unknown command: {cmd}");
            eprintln!("Usage: comboman [run] | new | list | delete <name>");
        }
    }
}
