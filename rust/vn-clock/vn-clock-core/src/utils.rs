//! Utility functions for the VN Clock.

use chrono::NaiveTime;

/// Parses a string into a `NaiveTime`.
/// Supports formats: HH, HH:MM, HH:MM:SS.
/// Special case: "24" or "24:00" is parsed as 00:00:00 (midnight).
pub fn parse_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim();
    if s == "24:00:00" || s == "24:00" || s == "24" {
        return NaiveTime::from_hms_opt(0, 0, 0);
    }
    
    if s.contains(':') {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() == 3 {
             return NaiveTime::parse_from_str(s, "%H:%M:%S").ok();
        } else if parts.len() == 2 {
             return NaiveTime::parse_from_str(s, "%H:%M").ok();
        }
    } else {
        // Just HH
        if s.len() <= 2 && s.chars().all(|c| c.is_ascii_digit()) {
            if let Ok(h) = s.parse::<u32>() {
                return NaiveTime::from_hms_opt(h, 0, 0);
            }
        }
    }

    None
}
