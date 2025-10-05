use crate::types::{Combo, SaveOption};
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use ratatui::widgets::ListState;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use std::io;
use std::ops::{Deref, DerefMut};

struct RawTerminal(Terminal<CrosstermBackend<io::Stdout>>);

impl Drop for RawTerminal {
    fn drop(&mut self) {
        disable_raw_mode().unwrap();
    }
}

impl Deref for RawTerminal {
    type Target = Terminal<CrosstermBackend<io::Stdout>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RawTerminal {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn setup_terminal() -> Result<RawTerminal, io::Error> {
    enable_raw_mode()?;
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(RawTerminal(terminal))
}

/// Stack-like selection: take a history (newest last) and return selected stack in order.
/// Controls:
///  - Up/Down: move cursor
///  - Space: toggle selection mode
///  - Left / d: remove current item from stack
///  - Enter: finalize, return Vec<String>
///  - Esc: cancel => None
pub fn select_stack(
    history: Vec<String>,
    initial_stack: Option<Vec<String>>,
) -> Option<Vec<String>> {
    if history.is_empty() {
        return None;
    }

    let mut terminal = setup_terminal().unwrap();
    let mut cursor_idx = history.len() - 1;
    let mut list_state = ListState::default();
    list_state.select(Some(cursor_idx));

    let mut selected_indices: Vec<usize> = if let Some(stack) = initial_stack {
        stack
            .into_iter()
            .filter_map(|s| history.iter().position(|h| h == &s))
            .collect()
    } else {
        vec![cursor_idx]
    };
    let mut selection_mode = true;
    if selected_indices.is_empty() {
        selected_indices.push(cursor_idx);
    }

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(size);

                let history_title = if selection_mode {
                    "History (Select Mode - SPACE for normal)"
                } else {
                    "History (Normal Mode - SPACE for select)"
                };

                // History pane: mark included commands with a check
                let history_items: Vec<ListItem> = history
                    .iter()
                    .enumerate()
                    .map(|(i, cmd)| {
                        let style = if selected_indices.contains(&i) {
                            Style::default().bg(Color::Blue)
                        } else {
                            Style::default()
                        };
                        ListItem::new(cmd.clone()).style(style)
                    })
                    .collect();

                let history_list = List::new(history_items)
                    .block(
                        Block::default()
                            .title(history_title)
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::Black)),
                    )
                    .highlight_style(Style::default().bg(Color::Yellow))
                    .highlight_symbol(">> ");

                f.render_stateful_widget(history_list, chunks[0], &mut list_state);

                // Selected stack pane
                let mut sorted_indices = selected_indices.clone();
                sorted_indices.sort();
                let stack_items: Vec<ListItem> = sorted_indices
                    .iter()
                    .map(|i| ListItem::new(history[*i].clone()))
                    .collect();
                let stack_list = List::new(stack_items).block(
                    Block::default()
                        .title("Selected Stack")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black)),
                );
                f.render_widget(stack_list, chunks[1]);
            })
            .unwrap();

        // Input
        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => {
                    if cursor_idx > 0 {
                        cursor_idx -= 1;
                        list_state.select(Some(cursor_idx));
                        if selection_mode && !selected_indices.contains(&cursor_idx) {
                            selected_indices.push(cursor_idx);
                        }
                    }
                }
                KeyCode::Down => {
                    if cursor_idx + 1 < history.len() {
                        cursor_idx += 1;
                        list_state.select(Some(cursor_idx));
                        if selection_mode && !selected_indices.contains(&cursor_idx) {
                            selected_indices.push(cursor_idx);
                        }
                    }
                }
                KeyCode::Char(' ') => {
                    selection_mode = !selection_mode;
                    if selection_mode {
                        if !selected_indices.contains(&cursor_idx) {
                            selected_indices.push(cursor_idx);
                        }
                    } else {
                        selected_indices.retain(|&i| i != cursor_idx);
                    }
                }
                KeyCode::Left | KeyCode::Char('d') => {
                    selected_indices.retain(|&i| i != cursor_idx);
                }
                KeyCode::Right => {
                    if !selected_indices.contains(&cursor_idx) {
                        selected_indices.push(cursor_idx);
                    }
                }
                KeyCode::Enter => {
                    selected_indices.sort();
                    let selected_stack: Vec<String> = selected_indices
                        .iter()
                        .map(|i| history[*i].clone())
                        .collect();
                    return Some(selected_stack);
                }
                KeyCode::Esc => return None,
                _ => {}
            }
        }
    }
}

pub fn select_save_option(stack: &[String]) -> Option<SaveOption> {
    let mut terminal = setup_terminal().unwrap();
    let mut list_state = ListState::default();
    list_state.select(Some(0));

    let options = [
        "Save as combo",
        "Save as function",
        "Save as script",
        "Edit",
    ];

    loop {
        terminal
            .draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(f.size());

                let stack_items: Vec<ListItem> =
                    stack.iter().map(|s| ListItem::new(s.as_str())).collect();
                let stack_list = List::new(stack_items).block(
                    Block::default()
                        .title("Selected Stack")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black)),
                );
                f.render_widget(stack_list, chunks[0]);

                let options_items: Vec<ListItem> =
                    options.iter().map(|&o| ListItem::new(o)).collect();
                let options_list = List::new(options_items)
                    .block(
                        Block::default()
                            .title("Save Options")
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::Black)),
                    )
                    .highlight_style(Style::default().bg(Color::Blue));
                f.render_stateful_widget(options_list, chunks[1], &mut list_state);
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => {
                    let i = list_state.selected().unwrap_or(0);
                    if i > 0 {
                        list_state.select(Some(i - 1));
                    }
                }
                KeyCode::Down => {
                    let i = list_state.selected().unwrap_or(0);
                    if i < options.len() - 1 {
                        list_state.select(Some(i + 1));
                    }
                }
                KeyCode::Enter => {
                    return match list_state.selected() {
                        Some(0) => Some(SaveOption::SaveAsCombo),
                        Some(1) => Some(SaveOption::SaveAsFunction),
                        Some(2) => Some(SaveOption::SaveAsScript),
                        Some(3) => Some(SaveOption::Edit),
                        _ => None,
                    };
                }
                KeyCode::Esc => return None,
                _ => {}
            }
        }
    }
}

/// UI for selecting an existing combo from `combos`.
/// Shows left pane list (filterable with fuzzy search) and right pane preview.
/// Typing filters; Backspace clears characters; Up/Down moves; Enter selects; Esc cancels.
pub fn run_ui(mut combos: Vec<Combo>) -> Option<String> {
    if combos.is_empty() {
        println!("No combos available.");
        return None;
    }

    // sort combos by last_used descending (most recent first)
    combos.sort_by(|a, b| b.last_used.cmp(&a.last_used));

    let matcher = SkimMatcherV2::default();
    let mut filtered = combos.clone();
    let mut selected_idx = 0usize;
    let mut list_state = ListState::default();
    list_state.select(Some(selected_idx));
    let mut filter_input = String::new();

    let mut terminal = setup_terminal().unwrap();

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
                    .split(size);

                let items: Vec<ListItem> = filtered
                    .iter()
                    .map(|c| ListItem::new(c.name.clone()))
                    .collect();
                let list = List::new(items)
                    .block(
                        Block::default()
                            .title(format!("Combos (filter: {filter_input})"))
                            .borders(Borders::ALL)
                            .style(Style::default().bg(Color::Black)),
                    )
                    .highlight_symbol(">>")
                    .highlight_style(Style::default().bg(Color::Blue));
                f.render_stateful_widget(list, chunks[0], &mut list_state);

                let preview_text = filtered
                    .get(selected_idx)
                    .map(|c| c.commands.join("\n"))
                    .unwrap_or_default();
                let preview = Paragraph::new(preview_text).block(
                    Block::default()
                        .title("Preview")
                        .borders(Borders::ALL)
                        .style(Style::default().bg(Color::Black)),
                );
                f.render_widget(preview, chunks[1]);
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Up => {
                    if selected_idx > 0 {
                        selected_idx -= 1;
                        list_state.select(Some(selected_idx));
                    }
                }
                KeyCode::Down => {
                    if selected_idx + 1 < filtered.len() {
                        selected_idx += 1;
                        list_state.select(Some(selected_idx));
                    }
                }
                KeyCode::Char(c) => {
                    filter_input.push(c);
                    filtered = combos
                        .iter()
                        .filter(|combo| matcher.fuzzy_match(&combo.name, &filter_input).is_some())
                        .cloned()
                        .collect();
                    selected_idx = 0;
                    list_state.select(Some(0));
                }
                KeyCode::Backspace => {
                    filter_input.pop();
                    filtered = combos
                        .iter()
                        .filter(|combo| matcher.fuzzy_match(&combo.name, &filter_input).is_some())
                        .cloned()
                        .collect();
                    selected_idx = 0;
                    list_state.select(Some(0));
                }
                KeyCode::Enter => {
                    return filtered.get(selected_idx).map(|c| c.name.clone());
                }
                KeyCode::Esc => return None,
                _ => {}
            }
        }
    }
}

pub fn prompt_input(prompt: &str) -> Option<String> {
    let mut terminal = setup_terminal().unwrap();
    let mut input = String::new();

    loop {
        terminal
            .draw(|f| {
                let size = f.size();
                let block = Block::default().style(Style::default().bg(Color::Black));
                f.render_widget(block, size);

                let prompt_text = format!("{prompt}{input}");
                let input_paragraph = Paragraph::new(prompt_text)
                    .block(Block::default().borders(Borders::ALL).title("Input"));
                f.render_widget(input_paragraph, size);
            })
            .unwrap();

        if let Event::Key(key) = event::read().unwrap() {
            match key.code {
                KeyCode::Char(c) => {
                    input.push(c);
                }
                KeyCode::Backspace => {
                    input.pop();
                }
                KeyCode::Enter => {
                    if input.is_empty() {
                        return None;
                    } else {
                        return Some(input);
                    }
                }
                KeyCode::Esc => return None,
                _ => {}
            }
        }
    }
}
