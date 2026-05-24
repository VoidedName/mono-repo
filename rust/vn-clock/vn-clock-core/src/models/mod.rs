pub mod color;

use chrono::{Duration, NaiveTime};
use serde::{Deserialize, Serialize};
pub use color::ClockColor;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEvent {
    pub time: NaiveTime,
    pub name: String,
    pub auto_pause: bool,
    pub repeat_interval: Option<Duration>,
    pub repeat_until: Option<NaiveTime>,
    pub color: ClockColor,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LogEntry {
    pub message: String,
    pub color: ClockColor,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClockConfig {
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub events: Vec<TimedEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClockState {
    pub clock_time: NaiveTime,
    pub initial_time: NaiveTime,
    pub target_speed: f64,
    pub paused: bool,
    pub events: Vec<TimedEvent>,
    pub logs: Vec<LogEntry>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClockEvent {
    TogglePause,
    SetTime(NaiveTime),
    SetSpeed(f64),
    AddTimedEvent(TimedEvent),
    RemoveTimedEvent(usize),
    LoadConfig(ClockConfig),
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
    pub clock_time: NaiveTime,
    pub initial_time: NaiveTime,
    pub speed: f64,
    pub paused: bool,
    pub events: Vec<TimedEvent>,
    pub logs: Vec<LogEntry>,
    pub last_tick: web_time::Instant,
    pub target_speed: f64,
    pub output_events: Vec<ClockOutputEvent>,
}
