mod types;
mod store;
mod ui;
mod exec;
mod history;

use std::env;
use std::io::{stdout, Write};

use crate::exec::{edit_stack, run_combo};
use crate::history::import_history;
use crate::store::{add_combo, load_combos, save_combos, update_last_used};
use crate::ui::{prompt_input, run_ui, select_save_option, select_stack};
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

fn delete_combo(combos: &mut Vec<Combo>, name: &str, combo_directory: Option<String>) {
    let before = combos.len();
    combos.retain(|c| c.name != name);
    if combos.len() < before {
        save_combos(combos, combo_directory);
        println!("Deleted combo '{name}'");
    } else {
        eprintln!("Combo '{name}' not found");
    }
}



use clap::Parser;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long, global = true)]
    combo_directory: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser)]
enum Commands {
    List,
    Delete { name: String },
    New,
    #[command(name = "run")]
    Run {
        name: Option<String>,
        #[arg(long)]
        no_confirm: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    let combo_dir = cli.combo_directory.clone();

    let mut combos = load_combos(cli.combo_directory);

    match cli.command {
        Commands::List => list_combos(&combos),
        Commands::Delete { name } => {
            delete_combo(&mut combos, &name, combo_dir);
        }
        Commands::New => {
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
                        if let Some(path) = prompt_input("Enter path to save script: ") {
                            let script = stack.join("\n");
                            if fs::write(&path, script).is_ok() {
                                println!("\nSaved script to {path}");
                            } else {
                                eprintln!("Failed to save script to {path}");
                            }
                        }
                        break;
                    }
                    Some(SaveOption::SaveAsFunction) => {
                        let name = prompt_input("Enter function name (leave blank for default): ");
                        let func_name =
                            name.unwrap_or_else(|| format!("command_{}", combos.len() + 1));
                        let function =
                            format!("\n{} () {{ \n{}\n }}\n", func_name, stack.join("\n"));

                        // Simplified: Append to .bashrc. A real implementation would need to be more robust.
                        let shell_rc = Path::new(&env::var("HOME").unwrap()).join(".bashrc");
                        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&shell_rc) {
                            if file.write_all(function.as_bytes()).is_ok() {
                                println!("\nAdded function '{func_name}' to your shell rc file.");
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
                            prompt_input("Enter name for combo (leave blank to auto-generate): ");
                        add_combo(&mut combos, stack, name, combo_dir.clone());
                        println!("\nCombo saved.");
                        break;
                    }
                    None => {
                        println!("Selection cancelled.");
                        break;
                    }
                }
            }
        }
        Commands::Run { name, no_confirm } => {
            // Interactive run UI
            if combos.is_empty() {
                println!("No saved combos. Use 'comboman new' to create one.");
                return;
            }
            let combo_name = match name {
                Some(n) => n,
                None => run_ui(combos.clone()).expect("No combo selected"),
            };

            if let Some(combo) = combos.iter().find(|c| c.name == combo_name) {
                if !no_confirm {
                    let confirm = prompt_input(&format!("Run combo '{}'? [Y/n] ", combo.name));
                    if confirm.as_deref() == Some("n") {
                        println!("Cancelled.");
                        return;
                    }
                }
                run_combo(combo);
                update_last_used(&mut combos, &combo_name);
                save_combos(&combos, combo_dir.clone());
            } else {
                eprintln!("Selected combo '{combo_name}' not found (concurrent modification?).");
            }
        }
    }
}
