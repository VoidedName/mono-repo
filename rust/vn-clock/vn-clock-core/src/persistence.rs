//! Serialization and deserialization utilities for clock configurations and states.

use crate::models::{ClockConfig, ClockState, CoreApp, ClockEvent};
use serde_json;

pub const CONFIG_EXTENSION: &str = "clockcfg";
pub const STATE_EXTENSION: &str = "clockstate";
pub const CONFIG_FILTER_NAME: &str = "Clock Configuration";
pub const STATE_FILTER_NAME: &str = "Clock Session State";

/// Serializes the current app configuration (initial time, speed, events).
pub fn serialize_config(app: &CoreApp) -> Result<String, serde_json::Error> {
    let config = ClockConfig {
        initial_time: app.initial_time(),
        target_speed: app.target_speed(),
        events: app.events().to_vec(),
    };
    serde_json::to_string_pretty(&config)
}

pub fn deserialize_config(json: &str) -> Result<ClockConfig, serde_json::Error> {
    serde_json::from_str(json)
}

/// Serializes the full application state (current time, speed, events, logs).
pub fn serialize_state(app: &CoreApp) -> Result<String, serde_json::Error> {
    let state = ClockState {
        clock_time: app.clock_time(),
        initial_time: app.initial_time(),
        target_speed: app.target_speed(),
        paused: app.paused(),
        events: app.events().to_vec(),
        logs: app.logs().to_vec(),
    };
    serde_json::to_string_pretty(&state)
}

pub fn deserialize_state(json: &str) -> Result<ClockState, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn get_config_extension() -> &'static str {
    CONFIG_EXTENSION
}

pub fn get_state_extension() -> &'static str {
    STATE_EXTENSION
}

/// Appends the specified extension to the filename if it's missing.
pub fn ensure_extension(filename: &str, extension: &str) -> String {
    let ext_with_dot = format!(".{}", extension);
    if filename.ends_with(&ext_with_dot) {
        filename.to_string()
    } else {
        format!("{}{}", filename, ext_with_dot)
    }
}

impl CoreApp {
    /// Serializes the configuration to a JSON string.
    pub fn get_config_json(&self) -> Result<String, String> {
        serialize_config(self).map_err(|e| format!("Failed to serialize config: {}", e))
    }

    /// Serializes the current state to a JSON string.
    pub fn get_state_json(&self) -> Result<String, String> {
        serialize_state(self).map_err(|e| format!("Failed to serialize state: {}", e))
    }

    /// Loads a configuration from a JSON string into the app.
    pub fn load_config_json(&mut self, json: &str) -> Result<(), String> {
        match deserialize_config(json) {
            Ok(config) => {
                self.handle_event(ClockEvent::LoadConfig(config));
                Ok(())
            }
            Err(e) => Err(format!("Failed to parse config: {}", e)),
        }
    }

    /// Loads a full session state from a JSON string into the app.
    pub fn load_state_json(&mut self, json: &str) -> Result<(), String> {
        match deserialize_state(json) {
            Ok(state) => {
                self.handle_event(ClockEvent::LoadState(state));
                Ok(())
            }
            Err(e) => Err(format!("Failed to parse state: {}", e)),
        }
    }
}
