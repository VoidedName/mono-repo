use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
    Frame,
};
use crate::models::{App, InputMode, Section};
use crate::models::color_serde::to_ratatui_color;
use vn_clock_core::models::ClockColor;

fn id_to_ratatui_color(id: u32) -> Color {
    to_ratatui_color(ClockColor::Dynamic(id))
}

pub const HELP_TEXT: &str = "
    SCROLLING:
    - Left/Right: Switch between Config Status and Event Log
    - Up/Down: Scroll selected section

    CLOCK CONTROLS:
    - Space: Pause/Resume
    - v: Set Speed (multiplier)
    - r: Reset to initial time
    - t: Set Time (HH:MM:SS)
    - e: Event Management Mode
    - S: Save Configuration (events + start time + speed)
    - L: Load Configuration
    - s: Save State (current time + events + speed)
    - l: Load State
    - h: Show/Hide Help
    - q: Quit

    EVENT MANAGEMENT:
    - a: Add new event
    - d: Delete selected event
    - Up/Down: Navigate list
    - Esc: Return to Clock view

    MISC:
    - Press 'h' or 'Esc' to close this menu.
    ";

pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.area());

    let status = match app.input_mode {
        InputMode::Normal => {
            if app.core.paused() {
                "PAUSED"
            } else {
                "RUNNING"
            }
        }
        InputMode::Help => "HELP - Press 'h' or 'Esc' to close",
        InputMode::InputFlow => {
            &app.input_flow.get_current_prompt()
        }
        InputMode::EventManagement => "EVENT MANAGEMENT",
        InputMode::LoadingConfig => "LOAD CONFIGURATION",
        InputMode::SavingConfig => "SAVE CONFIGURATION",
        InputMode::LoadingState => "LOAD STATE",
        InputMode::SavingState => "SAVE STATE",
        InputMode::ConfirmOverwriteConfig(_) | InputMode::ConfirmOverwriteState(_) => "CONFIRM OVERWRITE",
    };
    let clock_text = match app.input_mode {
        InputMode::Normal | InputMode::EventManagement | InputMode::Help => format!(
            "{} | {}",
            app.core.clock_time().format("%H:%M:%S%.3f"),
            status
        ),
        InputMode::InputFlow => format!("INPUT: {} | {}", app.input_flow.buffer, status),
        _ => format!("INPUT: {} | {}", app.input_flow.buffer, status),
    };
    let clock_para = Paragraph::new(clock_text)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title("Digital Clock"));
    f.render_widget(clock_para, chunks[0]);

    let lower_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let (config_title, config_block_style) = if app.selected_section == Section::Config {
        ("Configuration Status (SELECTED)", Style::default().bg(Color::Rgb(30, 30, 60)))
    } else {
        ("Configuration Status", Style::default())
    };

    match app.input_mode {
        InputMode::EventManagement => {
            let events: Vec<ListItem> = app.core
                .events()
                .iter()
                .enumerate()
                .map(|(i, e)| {
                    let text = format!("[{}] {}", e.id, e.to_display_string());
                    let style = if i == app.selected_event {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(id_to_ratatui_color(e.id))
                    };
                    ListItem::new(text).style(style)
                })
                .collect();
            let events_list =
                List::new(events).block(Block::default().borders(Borders::ALL).title("Configured Events").style(config_block_style));
            f.render_widget(events_list, lower_chunks[0]);
        }
        _ => {
            let mut config_lines = vec![
                ListItem::new(app.core.get_initial_time_string()),
                ListItem::new(app.core.get_target_speed_string()),
                ListItem::new("Events:"),
            ];

            if app.core.events().is_empty() {
                config_lines.push(ListItem::new("  (None)"));
            } else {
                for e in app.core.events().iter() {
                    let text = format!("  [{}] {}", e.id, e.to_display_string());
                    config_lines.push(ListItem::new(text).style(Style::default().fg(id_to_ratatui_color(e.id))));
                }
            }

            // Adjust scroll
            let max_scroll = config_lines.len().saturating_sub(1);
            let scroll = app.config_scroll.min(max_scroll);
            
            let scroll_indicator = if max_scroll > 0 {
                format!(" (Scroll: {}/{})", scroll, max_scroll)
            } else {
                "".to_string()
            };

            let visible_config: Vec<ListItem> = config_lines.into_iter().skip(scroll).collect();
            
            let config_list = List::new(visible_config)
                .block(Block::default().borders(Borders::ALL).title(format!("{}{}", config_title, scroll_indicator)).style(config_block_style));
            f.render_widget(config_list, lower_chunks[0]);
        }
    }

    let (log_title, log_block_style) = if app.selected_section == Section::Log {
        ("Event Log (SELECTED)", Style::default().bg(Color::Rgb(30, 30, 60)))
    } else {
        ("Event Log", Style::default())
    };

    let max_log_scroll = app.core.logs().len().saturating_sub(1);
    let log_scroll = app.log_scroll.min(max_log_scroll);
    
    let log_scroll_indicator = if max_log_scroll > 0 {
        format!(" (Scroll: {}/{})", log_scroll, max_log_scroll)
    } else {
        "".to_string()
    };

    let logs: Vec<ListItem> = app.core
        .logs()
        .iter()
        .rev()
        .skip(log_scroll)
        .map(|log| ListItem::new(log.message.as_str()).style(Style::default().fg(to_ratatui_color(log.color))))
        .collect();
    let logs_list = List::new(logs)
        .block(Block::default().borders(Borders::ALL).title(format!("{}{}", log_title, log_scroll_indicator)).style(log_block_style));
    f.render_widget(logs_list, lower_chunks[1]);

    // Footnotes or extra status
    let status_text = format!("Mode: {:<20}", app.input_flow.get_flow_name());

    let status_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(20)].as_ref())
        .split(chunks[2].inner(ratatui::layout::Margin { horizontal: 1, vertical: 1 }));

    let mode_para = Paragraph::new(status_text);
    let help_para = Paragraph::new("press h for help")
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .alignment(ratatui::layout::Alignment::Right);

    f.render_widget(Block::default().borders(Borders::ALL).title("Status"), chunks[2]);
    f.render_widget(mode_para, status_layout[0]);
    f.render_widget(help_para, status_layout[1]);

    if let InputMode::Help = app.input_mode {
        render_help_overlay(f, app);
    }

    match app.input_mode {
        InputMode::LoadingConfig | InputMode::SavingConfig | InputMode::LoadingState | InputMode::SavingState => {
            render_file_explorer(f, app);
        }
        InputMode::ConfirmOverwriteConfig(ref filename) | InputMode::ConfirmOverwriteState(ref filename) => {
            render_confirm_overwrite_dialog(f, filename);
        }
        _ => {}
    }
}

pub fn render_confirm_overwrite_dialog(f: &mut Frame, filename: &str) {
    let area = centered_rect(40, 20, f.area());
    let block = Block::default()
        .borders(Borders::ALL)
        .title("Overwrite Confirmation")
        .style(Style::default().fg(Color::Yellow));

    let text = format!("File '{}' already exists.\nOverwrite? (y/n)", filename);
    let para = Paragraph::new(text)
        .block(block)
        .alignment(ratatui::layout::Alignment::Center);

    f.render_widget(Clear, area);
    f.render_widget(para, area);
}

pub fn render_file_explorer(f: &mut Frame, app: &App) {
    let area = centered_rect(60, 60, f.area());
    let title = match app.input_mode {
        InputMode::LoadingConfig => "Load Config",
        InputMode::SavingConfig => "Save Config (Type name or select)",
        InputMode::LoadingState => "Load State",
        InputMode::SavingState => "Save State (Type name or select)",
        _ => "File Explorer",
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = app
        .files
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let style = if Some(i) == app.selected_file {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(name.as_str()).style(style)
        })
        .collect();

    let list = List::new(items).block(block).highlight_style(Style::default().add_modifier(Modifier::BOLD));
    
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(3)].as_ref())
        .split(area);

    f.render_widget(list, chunks[0]);

    let input_block = Block::default().borders(Borders::ALL).title("File Name");
    let input_para = Paragraph::new(app.input_flow.buffer.as_str()).block(input_block);
    f.render_widget(input_para, chunks[1]);
}

pub fn render_help_overlay(f: &mut Frame, app: &App) {
    let help_lines: Vec<&str> = HELP_TEXT.lines().collect();
    let total_lines = help_lines.len();
    
    // Calculate visible area height
    let area = centered_rect(60, 60, f.area());
    let visible_height = area.height.saturating_sub(2) as usize; // -2 for borders

    let scroll = app.help_scroll.min(total_lines.saturating_sub(visible_height));
    let visible_help_text = help_lines
        .iter()
        .skip(scroll)
        .take(visible_height)
        .cloned()
        .collect::<Vec<_>>()
        .join("\n");

    let scroll_indicator = if total_lines > visible_height {
        format!(" (Scroll: {}/{})", scroll, total_lines.saturating_sub(visible_height))
    } else {
        "".to_string()
    };

    let block = Block::default()
        .title(format!("Controls Help{}", scroll_indicator))
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::Yellow));

    f.render_widget(Clear, area); //this clears out the background
    f.render_widget(Paragraph::new(visible_help_text).block(block), area);
}

pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}
