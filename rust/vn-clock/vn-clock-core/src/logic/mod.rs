use chrono::{Duration, NaiveTime, Timelike};
use crate::models::{CoreApp, LogEntry, ClockColor, ClockEvent, ClockOutputEvent};
use web_time::Instant;

impl CoreApp {
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
            ClockEvent::AddTimedEvent(event) => {
                self.add_log(format!("Added event: {}", event.name), ClockColor::White);
                self.events.push(event);
            }
            ClockEvent::RemoveTimedEvent(index) => {
                if index < self.events.len() {
                    let removed = self.events.remove(index);
                    self.add_log(format!("Removed event: {}", removed.name), ClockColor::White);
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

    pub fn add_log(&mut self, message: String, color: ClockColor) {
        let entry = LogEntry { message, color };
        self.logs.push(entry.clone());
        self.output_events.push(ClockOutputEvent::Log(entry));
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
            t2 += 86400;
        }

        let events = self.events.clone();

        for event in events {
            let base_t = event.time.num_seconds_from_midnight() as i64;

            let mut trigger_times = vec![base_t];
            if let Some(interval) = event.repeat_interval {
                let period = interval.num_seconds();
                if period > 0 {
                    let mut until_t = event.repeat_until
                        .map(|t| t.num_seconds_from_midnight() as i64)
                        .unwrap_or(86399);
                    
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
                    self.output_events.push(ClockOutputEvent::Ding);
                    if event.auto_pause {
                        self.paused = true;
                        self.output_events.push(ClockOutputEvent::Paused(true));
                    }
                }
            }
        }
    }
}
