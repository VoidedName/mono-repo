use chrono::NaiveTime;

pub fn parse_time(s: &str) -> Option<NaiveTime> {
    let s = s.trim();
    if s == "24:00:00" || s == "24:00" || s == "24" {
        return NaiveTime::from_hms_opt(0, 0, 0);
    }
    if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M:%S") {
        Some(time)
    } else if let Ok(time) = NaiveTime::parse_from_str(s, "%H:%M") {
        Some(time)
    } else if let Ok(time) = NaiveTime::parse_from_str(s, "%H") {
        Some(time)
    } else {
        None
    }
}
