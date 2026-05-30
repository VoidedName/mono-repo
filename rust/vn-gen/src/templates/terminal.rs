use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generates the Terminal (Ratatui) target crate.
pub fn create(root: &Path, name: &str) -> Result<()> {
    let crate_name = format!("{}-terminal", name);
    let path = root.join(&crate_name);
    fs::create_dir_all(path.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
{name}-core = {{ path = "../{name}-core" }}
anyhow = {{ workspace = true }}
ratatui = "0.29.0"
crossterm = {{ version = "0.28.1", features = ["events"] }}
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    let crate_safe_name = name.replace('-', "_");
    let main_rs = format!(
        r#"use anyhow::Result;
use {crate_safe_name}_core::{{Counter, PlatformHooks}};
use crossterm::{{
    event::{{self, Event, KeyCode, KeyEventKind}},
    execute,
    terminal::{{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen}},
}};
use ratatui::{{
    backend::CrosstermBackend,
    layout::{{Constraint, Direction, Layout}},
    widgets::{{Block, Borders, List, ListItem, Paragraph}},
    Terminal,
}};
use std::io;

struct TerminalHooks;
impl PlatformHooks for TerminalHooks {{}}

fn main() -> Result<()> {{
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut counter = Counter::new();
    let hooks = TerminalHooks;

    loop {{
        terminal.draw(|f| {{
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(f.area());

            let title = Paragraph::new(format!("Count: {{}}", counter.count()))
                .block(Block::default().borders(Borders::ALL).title("Counter"));
            f.render_widget(title, chunks[0]);

            let logs: Vec<ListItem> = counter
                .logs()
                .iter()
                .rev()
                .map(|l| ListItem::new(format!("[{{}}] {{}}", l.timestamp.format("%H:%M:%S"), l.message)))
                .collect();
            let logs_list = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));
            f.render_widget(logs_list, chunks[1]);
        }})?;

        if event::poll(std::time::Duration::from_millis(16))? {{
            if let Event::Key(key) = event::read()? {{
                if key.kind == KeyEventKind::Press {{
                    match key.code {{
                        KeyCode::Char('q') => break,
                        KeyCode::Char('+') => counter.increment(&hooks),
                        KeyCode::Char('-') => counter.decrement(&hooks),
                        KeyCode::Char('r') => counter.reset(&hooks),
                        _ => {{}}
                    }}
                }}
            }}
        }}
    }}

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    Ok(())
}}
"#
    );
    fs::write(path.join("src").join("main.rs"), main_rs)?;
    Ok(())
}
