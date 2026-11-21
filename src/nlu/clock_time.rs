//! Clock speech uses Home Assistant's timezone when the integration has synced
//! it, then the process timezone (`TZ`), then UTC.

use jiff::tz::TimeZone;
use jiff::Timestamp;

pub(super) fn hhmm_now(zone_name: &str) -> Option<String> {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    hhmm_at(i64::try_from(secs).ok()?, zone_name)
}

pub(super) fn hhmm_at(unix_secs: i64, zone_name: &str) -> Option<String> {
    let stamp = Timestamp::from_second(unix_secs).ok()?;
    let zoned = stamp.to_zoned(resolve_zone(zone_name));
    let time = zoned.time();
    Some(format!("{:02}:{:02}", time.hour(), time.minute()))
}

fn resolve_zone(zone_name: &str) -> TimeZone {
    let name = zone_name.trim();
    if !name.is_empty() {
        if let Ok(zone) = TimeZone::get(name) {
            return zone;
        }
    }
    TimeZone::system()
}

#[cfg(test)]
mod tests {
    use super::hhmm_at;

    #[test]
    fn new_york_in_november_is_five_hours_behind_utc() {
        // 2026-11-21 15:06:00 UTC is 10:06 EST.
        let stamp = 1_795_273_560;
        assert_eq!(hhmm_at(stamp, "America/New_York").as_deref(), Some("10:06"));
        assert_eq!(hhmm_at(stamp, "UTC").as_deref(), Some("15:06"));
    }

    #[test]
    fn unknown_zone_name_matches_an_empty_zone() {
        let stamp = 1_795_273_560;
        assert_eq!(hhmm_at(stamp, "Not/A_Zone").as_deref(), hhmm_at(stamp, "").as_deref());
    }
}
