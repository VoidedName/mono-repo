use chrono::{Duration, NaiveTime, Timelike};
use crate::models::{CoreApp, LogEntry, ClockColor, ClockEvent, ClockOutputEvent, TimedEvent};
use web_time::Instant;

impl CoreApp {
    /// Creates a new `CoreApp` with default settings (midnight, paused, 1.0x speed).
    pub fn new() -> Self {
        Self {
            clock_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            initial_time: NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
            speed: 0.0,
            paused: true,
            events: vec![],
            logs: vec![LogEntry {
                message: "Welcome to Digital Clock!".to_string(),
                color: ClockColor::White,
            }],
            last_tick: Instant::now(),
            target_speed: 1.0,
            output_events: vec![],
        }
    }

    /// Processes a `ClockEvent` and updates the internal state accordingly.
    pub fn handle_event(&mut self, event: ClockEvent) {
        match event {
            ClockEvent::TogglePause => {
                self.paused = !self.paused;
                self.output_events.push(ClockOutputEvent::Paused(self.paused));
                let status = if self.paused { "Paused" } else { "Resumed" };
                self.add_log(status.to_string(), ClockColor::White);
            }
            ClockEvent::SetTime(time) => {
                self.clock_time = time;
                self.output_events.push(ClockOutputEvent::TimeSet(time));
                self.add_log(format!("Time set to {}", time.format("%H:%M:%S")), ClockColor::White);
            }
            ClockEvent::SetSpeed(speed) => {
                self.target_speed = speed;
                self.output_events.push(ClockOutputEvent::SpeedSet(speed));
                self.add_log(format!("Speed set to {:.2}x", speed), ClockColor::White);
            }
            ClockEvent::AddTimedEvent(config) => {
                let id = self.get_lowest_available_id();
                self.add_log(format!("Added event: {} (ID: {})", config.name, id), ClockColor::White);
                self.events.push(TimedEvent { id, config });
            }
            ClockEvent::RemoveTimedEvent(id) => {
                if let Some(pos) = self.events.iter().position(|e| e.id == id) {
                    let removed = self.events.remove(pos);
                    self.add_log(format!("Removed event: {} (ID: {})", removed.config.name, id), ClockColor::White);
                }
            }
            ClockEvent::LoadConfig(config) => {
                self.clock_time = config.initial_time;
                self.initial_time = config.initial_time;
                self.target_speed = config.target_speed;
                self.events = config.events;
                self.add_log("Configuration loaded".to_string(), ClockColor::White);
                self.output_events.push(ClockOutputEvent::TimeSet(self.clock_time));
                self.output_events.push(ClockOutputEvent::SpeedSet(self.target_speed));
            }
            ClockEvent::LoadState(state) => {
                self.clock_time = state.clock_time;
                self.initial_time = state.initial_time;
                self.target_speed = state.target_speed;
                self.paused = state.paused;
                self.events = state.events.clone();
                self.logs = state.logs.clone();
                self.add_log("State loaded".to_string(), ClockColor::White);
                self.output_events.push(ClockOutputEvent::TimeSet(self.clock_time));
                self.output_events.push(ClockOutputEvent::SpeedSet(self.target_speed));
                self.output_events.push(ClockOutputEvent::Paused(self.paused));
            }
            ClockEvent::Reset => {
                self.clock_time = self.initial_time;
                self.paused = true;
                self.output_events.push(ClockOutputEvent::TimeSet(self.clock_time));
                self.output_events.push(ClockOutputEvent::Paused(self.paused));
                self.add_log("Clock reset".to_string(), ClockColor::White);
            }
        }
    }

    /// Finds the smallest non-negative integer not currently used as an event ID.
    fn get_lowest_available_id(&self) -> u32 {
        let mut id = 0;
        loop {
            if !self.events.iter().any(|e| e.id == id) {
                return id;
            }
            id += 1;
        }
    }

    /// Appends a new message to the internal log and emits a `Log` output event.
    pub fn add_log(&mut self, message: String, color: ClockColor) {
        let entry = LogEntry { message, color };
        self.logs.push(entry.clone());
        self.output_events.push(ClockOutputEvent::Log(entry));
    }

    /// Progresses the clock state based on the time elapsed since the last call.
    pub fn tick(&mut self) {
        let now = Instant::now();
        let delta = now.duration_since(self.last_tick);
        self.last_tick = now;

        if self.paused {
            self.speed = 0.0;
            return;
        }

        self.speed = self.target_speed;
        let old_time = self.clock_time;
        self.clock_time = self.add_delta_to_time(self.clock_time, delta.as_secs_f64() * self.speed);
        self.check_events(old_time, self.clock_time);
    }

    /// Robustly adds fractional seconds to a `NaiveTime`, handling midnight rollovers.
    fn add_delta_to_time(&self, time: NaiveTime, seconds_to_add: f64) -> NaiveTime {
        let duration_to_add = Duration::nanoseconds((seconds_to_add * 1_000_000_000.0) as i64);
        
        let total_nanos = (time.num_seconds_from_midnight() as i64 * 1_000_000_000 + time.nanosecond() as i64) 
            + duration_to_add.num_nanoseconds().unwrap_or(0);
        
        let secs = (total_nanos / 1_000_000_000) % (24 * 3600);
        let nanos = total_nanos % 1_000_000_000;
        
        let secs = if secs < 0 { secs + 86400 } else { secs };
        let nanos = if nanos < 0 { nanos + 1_000_000_000 } else { nanos };

        NaiveTime::from_num_seconds_from_midnight_opt(secs as u32, nanos as u32).unwrap()
    }

    /// Checks if any events should trigger between `old_time` and `new_time`.
    /// Sorts triggered events chronologically before processing.
    pub fn check_events(&mut self, old_time: NaiveTime, new_time: NaiveTime) {
        let t1 = old_time.num_seconds_from_midnight() as i64;
        let mut t2 = new_time.num_seconds_from_midnight() as i64;

        if t2 < t1 {
            t2 += 86400;
        }

        let events = self.events.clone();
        let mut triggered_events = Vec::new();

        for event in events {
            let base_t = event.config.time.num_seconds_from_midnight() as i64;

            let mut trigger_times = vec![base_t];
            if let Some(interval) = event.config.repeat_interval {
                let period = interval.num_seconds();
                if period > 0 {
                    let mut until_t = event.config.repeat_until
                        .map(|t| t.num_seconds_from_midnight() as i64)
                        .unwrap_or(86399);
                    
                    if event.config.repeat_until.is_some() && until_t <= base_t {
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
                for offset in &[-86400, 0, 86400] {
                    let adjusted_tt = tt + offset;
                    if adjusted_tt > t1 && adjusted_tt <= t2 {
                        triggered_events.push((adjusted_tt, event.clone()));
                        break;
                    }
                }
            }
        }

        // Sort by trigger time
        triggered_events.sort_by_key(|(t, _)| *t);

        for (tt, event) in triggered_events {
            // Normalize trigger time back to 0-86400 range for display
            let display_t = ((tt % 86400) + 86400) % 86400;
            let trigger_time_str = NaiveTime::from_num_seconds_from_midnight_opt(display_t as u32, 0)
                .map(|t| t.format("%H:%M:%S").to_string())
                .unwrap_or_else(|| "??:??:??".to_string());

            self.add_log(
                format!(
                    "[{}] EVENT: {} (Scheduled for {})",
                    self.clock_time.format("%H:%M:%S"),
                    event.config.name,
                    trigger_time_str
                ),
                ClockColor::from_id(event.id),
            );
            self.output_events.push(ClockOutputEvent::Ding);
            if event.config.auto_pause {
                self.paused = true;
                self.output_events.push(ClockOutputEvent::Paused(true));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TimedEventConfig;

    #[test]
    fn test_id_management() {
        let mut app = CoreApp::new();
        
        // Add 3 events
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Event 0".to_string(),
            time: NaiveTime::from_hms_opt(1, 0, 0).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Event 1".to_string(),
            time: NaiveTime::from_hms_opt(2, 0, 0).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Event 2".to_string(),
            time: NaiveTime::from_hms_opt(3, 0, 0).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));
        
        assert_eq!(app.events[0].id, 0);
        assert_eq!(app.events[1].id, 1);
        assert_eq!(app.events[2].id, 2);
        
        // Remove ID 1
        app.handle_event(ClockEvent::RemoveTimedEvent(1));
        assert_eq!(app.events.len(), 2);
        assert_eq!(app.events[0].id, 0);
        assert_eq!(app.events[1].id, 2);
        
        // Add new event, should get ID 1
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Event New".to_string(),
            time: NaiveTime::from_hms_opt(4, 0, 0).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));
        
        assert!(app.events.iter().any(|e| e.id == 1));
        assert_eq!(app.events.len(), 3);
    }

    #[test]
    fn test_event_sorting_and_logging() {
        let mut app = CoreApp::new();
        // Time is 00:00:00
        app.clock_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        
        // Add event at 00:00:10
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Later".to_string(),
            time: NaiveTime::from_hms_opt(0, 0, 10).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));
        
        // Add event at 00:00:05
        app.handle_event(ClockEvent::AddTimedEvent(TimedEventConfig {
            name: "Earlier".to_string(),
            time: NaiveTime::from_hms_opt(0, 0, 5).unwrap(),
            auto_pause: false,
            repeat_interval: None,
            repeat_until: None,
        }));

        // Tick from 00:00:00 to 00:00:15
        let old_time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
        let new_time = NaiveTime::from_hms_opt(0, 0, 15).unwrap();
        
        // Clear logs from AddTimedEvent
        app.logs.clear();
        
        app.check_events(old_time, new_time);
        
        // Logs should be sorted: Earlier then Later
        assert_eq!(app.logs.len(), 2);
        assert!(app.logs[0].message.contains("Earlier"));
        assert!(app.logs[0].message.contains("Scheduled for 00:00:05"));
        assert!(app.logs[1].message.contains("Later"));
        assert!(app.logs[1].message.contains("Scheduled for 00:00:10"));
    }
}
