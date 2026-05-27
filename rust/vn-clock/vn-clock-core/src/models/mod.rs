pub mod color;

use chrono::{Duration, NaiveTime};
use serde::{Deserialize, Serialize};
pub use color::ClockColor;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimedEventConfig {
    pub time: NaiveTime,
    pub name: String,
    pub auto_pause: bool,
    pub repeat_interval: Option<Duration>,
    pub repeat_until: Option<NaiveTime>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TimedEvent {
    pub id: u32,
    pub config: TimedEventConfig,
}

impl TimedEvent {
    pub fn to_display_string(&self) -> String {
        let mut text = format!("{} at {}", self.config.name, self.config.time.format("%H:%M:%S"));
        if self.config.auto_pause {
            text.push_str(" (Auto-pause)");
        }
        if let Some(interval) = self.config.repeat_interval {
            let total_secs = interval.num_seconds();
            let hours = total_secs / 3600;
            let minutes = (total_secs % 3600) / 60;
            let seconds = total_secs % 60;
            text.push_str(&format!(" | Every {:02}:{:02}:{:02}", hours, minutes, seconds));
            if let Some(until) = self.config.repeat_until {
                let until_str = if until == NaiveTime::from_hms_opt(0, 0, 0).unwrap() {
                    "24:00:00".to_string()
                } else {
                    until.format("%H:%M:%S").to_string()
                };
                text.push_str(&format!(" until {}", until_str));
            }
        }
        text
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct LogEntry {
    pub message: String,
    pub color: ClockColor,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClockConfig {
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub events: Vec<TimedEvent>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ClockState {
    pub clock_time: NaiveTime,
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub paused: bool,
    pub events: Vec<TimedEvent>,
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum ClockEvent {
    TogglePause,
    SetTime(NaiveTime),
    SetSpeed(f64),
    AddTimedEvent(TimedEventConfig),
    RemoveTimedEvent(u32),
    LoadConfig(ClockConfig),
    LoadState(ClockState),
    Reset,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClockOutputEvent {
    Ding,
    Log(LogEntry),
    Paused(bool),
    TimeSet(NaiveTime),
    SpeedSet(f64),
}

pub struct CoreApp {
    pub(crate) clock_time: NaiveTime,
    pub(crate) initial_time: NaiveTime,
    pub(crate) speed: f64,
    pub(crate) paused: bool,
    pub(crate) events: Vec<TimedEvent>,
    pub(crate) logs: Vec<LogEntry>,
    pub(crate) last_tick: web_time::Instant,
    pub(crate) target_speed: f64,
    pub(crate) output_events: Vec<ClockOutputEvent>,
}

impl CoreApp {
    pub fn get_initial_time_string(&self) -> String {
        format!("Initial Time: {}", self.initial_time.format("%H:%M:%S"))
    }

    pub fn get_target_speed_string(&self) -> String {
        format!("Target Speed: {:.2}x", self.target_speed)
    }

    pub fn take_output_events(&mut self) -> Vec<ClockOutputEvent> {
        std::mem::take(&mut self.output_events)
    }

    pub fn clock_time(&self) -> NaiveTime {
        self.clock_time
    }

    pub fn initial_time(&self) -> NaiveTime {
        self.initial_time
    }

    pub fn speed(&self) -> f64 {
        self.speed
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn events(&self) -> &[TimedEvent] {
        &self.events
    }

    pub fn logs(&self) -> &[LogEntry] {
        &self.logs
    }

    pub fn target_speed(&self) -> f64 {
        self.target_speed
    }
}
