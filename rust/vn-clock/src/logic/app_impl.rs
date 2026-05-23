use chrono::{Duration, NaiveTime, Timelike};
use ratatui::style::Color;
use rodio::source::Source;
use std::{
    fs,
    sync::{Arc, Mutex},
    time::Instant,
};
use crate::models::{App, LogEntry, InputMode, Section};

impl App {
    pub fn new() -> Self {
        Self {
            clock_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            initial_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            speed: 0.0,
            paused: true,
            events: vec![],
            logs: vec![LogEntry {
                message: "Welcome to Digital Clock!".to_string(),
                color: Color::White,
            }],
            last_tick: Instant::now(),
            sink: Arc::new(Mutex::new(None)),
            input_mode: InputMode::Normal,
            input_buffer: String::new(),
            target_speed: 1.0,
            selected_event: 0,
            temp_event_name: String::new(),
            temp_event_time: None,
            temp_event_auto_pause: false,
            temp_event_repeat_interval: None,
            files: vec![],
            selected_file: None,
            config_scroll: 0,
            log_scroll: 0,
            help_scroll: 0,
            selected_section: Section::Log,
        }
    }

    pub fn add_log(&mut self, message: String, color: Color) {
        self.logs.push(LogEntry { message, color });
    }

    pub fn get_random_color(&self) -> Color {
        let colors = [
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::LightRed,
            Color::LightGreen,
            Color::LightYellow,
            Color::LightBlue,
            Color::LightMagenta,
            Color::LightCyan,
        ];
        // Use events length as a pseudo-random seed
        colors[self.events.len() % colors.len()]
    }

    pub fn refresh_files(&mut self, suffix: &str) {
        self.files.clear();
        if let Ok(entries) = fs::read_dir(".") {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.ends_with(suffix) {
                                self.files.push(name.to_string());
                            }
                        }
                    }
                }
            }
        }
        self.files.sort();
        self.selected_file = None;
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.last_tick = now;

        if self.paused {
            self.speed = 0.0;
            return;
        }

        self.speed = self.target_speed;

        let seconds_to_add = delta.as_secs_f64() * self.speed;
        let old_time = self.clock_time;
        
        let duration_to_add = Duration::nanoseconds((seconds_to_add * 1_000_000_000.0) as i64);
        
        let new_time_total_nanos = (self.clock_time.num_seconds_from_midnight() as i64 * 1_000_000_000 + self.clock_time.nanosecond() as i64) + duration_to_add.num_nanoseconds().unwrap();
        
        let secs = (new_time_total_nanos / 1_000_000_000) % (24 * 3600);
        let nanos = new_time_total_nanos % 1_000_000_000;
        
        self.clock_time = NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos as u32).unwrap();

        self.check_events(old_time, self.clock_time);
    }

    pub fn check_events(&mut self, old_time: NaiveTime, new_time: NaiveTime) {
        let t1 = old_time.num_seconds_from_midnight() as i64;
        let mut t2 = new_time.num_seconds_from_midnight() as i64;

        if t2 < t1 {
            // Clock wrapped around midnight during this tick
            t2 += 86400;
        }

        // Clone events to avoid borrowing issues while iterating and modifying self (logs, paused)
        let events = self.events.clone();

        for event in events {
            let base_t = event.time.num_seconds_from_midnight() as i64;

            let mut trigger_times = vec![base_t];
            if let Some(interval) = event.repeat_interval {
                let period = interval.num_seconds();
                if period > 0 {
                    let mut until_t = event.repeat_until
                        .map(|t| t.num_seconds_from_midnight() as i64)
                        .unwrap_or(86399); // Default to end of day if not specified
                    
                    // If until time is before or equal to start time, it refers to the next day
                    if event.repeat_until.is_some() && until_t <= base_t {
                        until_t += 86400;
                    }

                    let mut curr = base_t + period;
                    while curr <= until_t {
                        trigger_times.push(curr);
                        curr += period;
                    }
                }
            }

            for tt in trigger_times {
                // Check if this specific occurrence (tt) falls within our current tick interval (t1, t2]
                // We check tt, tt + 86400, and tt - 86400 to handle any wrap-around scenarios
                let mut triggered = false;
                for offset in &[-86400, 0, 86400] {
                    let adjusted_tt = tt + offset;
                    if adjusted_tt > t1 && adjusted_tt <= t2 {
                        triggered = true;
                        break;
                    }
                }

                if triggered {
                    self.add_log(
                        format!(
                            "[{}] EVENT: {}",
                            self.clock_time.format("%H:%M:%S"),
                            event.name
                        ),
                        event.color,
                    );
                    self.play_ding();
                    if event.auto_pause {
                        self.paused = true;
                    }
                }
            }
        }
    }

    pub fn play_ding(&self) {
        if let Ok(sink_guard) = self.sink.lock() {
            if let Some(sink) = sink_guard.as_ref() {
                // Play a simple beep
                let source = rodio::source::SineWave::new(440.0)
                    .take_duration(std::time::Duration::from_millis(200))
                    .amplify(0.2);
                sink.append(source);
            }
        }
    }
}
