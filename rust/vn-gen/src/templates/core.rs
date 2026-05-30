use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generates the shared core logic crate.
pub fn create(root: &Path, name: &str) -> Result<()> {
    let crate_name = format!("{}-core", name);
    let path = root.join(&crate_name);
    fs::create_dir_all(path.join("src"))?;

    let cargo_toml = format!(
        r#"[package]
name = "{crate_name}"
version = "0.1.0"
edition = "2024"

[dependencies]
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
anyhow = {{ workspace = true }}
web-time = {{ workspace = true }}
chrono = {{ version = "0.4", features = ["serde"] }}
"#
    );
    fs::write(path.join("Cargo.toml"), cargo_toml)?;

    let lib_rs = r#"use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use web_time::SystemTime;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub message: String,
}

pub trait PlatformHooks {
    fn now(&self) -> DateTime<Utc> {
        let now = SystemTime::now();
        let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        DateTime::from_timestamp(duration.as_secs() as i64, duration.subsec_nanos()).unwrap_or_default()
    }
}

pub struct Counter {
    count: i32,
    logs: Vec<LogEntry>,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            count: 0,
            logs: vec![LogEntry {
                timestamp: Utc::now(),
                message: "Counter initialized".to_string(),
            }],
        }
    }

    pub fn increment(&mut self, hooks: &impl PlatformHooks) {
        self.count += 1;
        self.add_log(hooks, format!("Incremented to {}", self.count));
    }

    pub fn decrement(&mut self, hooks: &impl PlatformHooks) {
        self.count -= 1;
        self.add_log(hooks, format!("Decremented to {}", self.count));
    }

    pub fn reset(&mut self, hooks: &impl PlatformHooks) {
        self.count = 0;
        self.add_log(hooks, "Reset counter".to_string());
    }

    pub fn count(&self) -> i32 {
        self.count
    }

    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }

    fn add_log(&mut self, hooks: &impl PlatformHooks, message: String) {
        self.logs.push(LogEntry {
            timestamp: hooks.now(),
            message,
        });
        if self.logs.len() > 50 {
            self.logs.remove(0);
        }
    }
}
"#;
    fs::write(path.join("src").join("lib.rs"), lib_rs)?;
    Ok(())
}
