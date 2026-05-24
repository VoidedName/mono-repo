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

#[derive(Serialize, Deserialize, Debug, Clone)]
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
