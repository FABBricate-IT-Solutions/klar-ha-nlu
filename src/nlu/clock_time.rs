//! Clock and calendar-date speech uses Home Assistant's timezone when the
//! integration has synced it, then the process timezone (`TZ`), then UTC.

use jiff::civil::Date;
use jiff::tz::TimeZone;
use jiff::Timestamp;

use super::context::ParseContext;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LocalStamp {
    pub hhmm: String,
    pub weekday: String,
    pub date: String,
}

pub(super) fn stamp_for(context: &ParseContext<'_>) -> Option<LocalStamp> {
    let pack = crate::lang::speech_lang().code();
    if let Some(secs) = context.settings.clock_unix {
        return stamp_at(secs, &context.settings.time_zone, pack);
    }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    stamp_at(i64::try_from(secs).ok()?, &context.settings.time_zone, pack)
}

#[cfg(test)]
pub(super) fn hhmm_at(unix_secs: i64, zone_name: &str) -> Option<String> {
    stamp_at(unix_secs, zone_name, "").map(|stamp| stamp.hhmm)
}

pub(super) fn stamp_at(unix_secs: i64, zone_name: &str, pack: &str) -> Option<LocalStamp> {
    let stamp = Timestamp::from_second(unix_secs).ok()?;
    let zoned = stamp.to_zoned(resolve_zone(zone_name));
    let time = zoned.time();
    let date = zoned.date();
    Some(LocalStamp {
        hhmm: format!("{:02}:{:02}", time.hour(), time.minute()),
        weekday: weekday_name(pack, date).to_string(),
        date: date_text(pack, date),
    })
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

fn german_family(pack: &str) -> bool {
    let pack = pack.to_ascii_lowercase();
    pack == "de" || pack.starts_with("de-") || pack.starts_with("de_")
}

fn weekday_name(pack: &str, date: Date) -> &'static str {
    // jiff weekday: Monday = 1 … Sunday = 7
    let mon0 = (date.weekday().to_monday_one_offset() as u8).wrapping_sub(1);
    if german_family(pack) {
        ["Montag", "Dienstag", "Mittwoch", "Donnerstag", "Freitag", "Samstag", "Sonntag"][mon0 as usize]
    } else {
        ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"][mon0 as usize]
    }
}

fn date_text(pack: &str, date: Date) -> String {
    let day = date.day() as u8;
    let month = date.month() as u8;
    let year = date.year();
    if german_family(pack) {
        let month_name =
            ["Januar", "Februar", "März", "April", "Mai", "Juni", "Juli", "August", "September", "Oktober", "November", "Dezember"]
                [(month - 1) as usize];
        format!("{day}. {month_name} {year}")
    } else {
        let month_name =
            ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"]
                [(month - 1) as usize];
        format!("{month_name} {}, {year}", ordinal(day))
    }
}

fn ordinal(day: u8) -> String {
    let suffix = if matches!(day % 100, 11..=13) {
        "th"
    } else {
        match day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        }
    };
    format!("{day}{suffix}")
}

#[cfg(test)]
mod tests {
    use super::{hhmm_at, stamp_at};

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

    #[test]
    fn berlin_monday_speaks_german_weekday() {
        // 2026-11-23 12:05:00 UTC = 13:05 Europe/Berlin, a Monday.
        let stamp = stamp_at(1_795_435_500, "Europe/Berlin", "de").expect("stamp");
        assert_eq!(stamp.hhmm, "13:05");
        assert_eq!(stamp.weekday, "Montag");
        assert!(stamp.date.contains("23"), "{}", stamp.date);
    }

    #[test]
    fn berlin_monday_speaks_english_weekday() {
        let stamp = stamp_at(1_795_435_500, "Europe/Berlin", "en").expect("stamp");
        assert_eq!(stamp.weekday, "Monday");
        assert!(stamp.date.contains("November"), "{}", stamp.date);
    }
}
