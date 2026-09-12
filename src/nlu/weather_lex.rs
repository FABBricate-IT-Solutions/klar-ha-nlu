//! Shared weather ask lexicon. Packs stay untouched; rain and umbrella match alone.

use crate::parse::normalize::fold_umlaut;

use super::weather_words::{
    AFTERNOON, AMBIG_UMBRELLA, DAYS, EVENING, FORECAST, FUTURE, GREET_STRIP, MORNING, NEED, NIGHT, RAIN, TODAY, TOMORROW, UMBRELLA,
    WEATHER, WEEKEND,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WeatherAsk {
    Now,
    Today,
    Tomorrow,
    Rain,
    Umbrella,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DayHint {
    Current,
    Today,
    Tomorrow,
    Weekend,
    Weekday(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum DayPart {
    Morning,
    Afternoon,
    Evening,
    Night,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WeatherClass {
    pub ask: WeatherAsk,
    pub day: DayHint,
    pub part: Option<DayPart>,
}

impl WeatherAsk {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Now => "now",
            Self::Today => "today",
            Self::Tomorrow => "tomorrow",
            Self::Rain => "rain",
            Self::Umbrella => "umbrella",
        }
    }

    pub(super) fn skips_climate(self) -> bool {
        matches!(self, Self::Rain | Self::Umbrella)
    }
}

impl DayHint {
    pub(super) fn as_str(self) -> Option<&'static str> {
        Some(match self {
            Self::Current => return None,
            Self::Today => "today",
            Self::Tomorrow => "tomorrow",
            Self::Weekend => "weekend",
            Self::Weekday(0) => "mon",
            Self::Weekday(1) => "tue",
            Self::Weekday(2) => "wed",
            Self::Weekday(3) => "thu",
            Self::Weekday(4) => "fri",
            Self::Weekday(5) => "sat",
            Self::Weekday(6) => "sun",
            Self::Weekday(_) => return None,
        })
    }
}

impl DayPart {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Morning => "morning",
            Self::Afternoon => "afternoon",
            Self::Evening => "evening",
            Self::Night => "night",
        }
    }
}

const QUERY_STOP: &[&str] = &[
    "what", "whats", "which", "how", "about", "the", "a", "an", "is", "it", "please", "wie", "was", "ist", "das", "der", "die", "den",
    "dem", "ein", "eine", "hoe", "wat", "het", "een", "quel", "quelle", "comment", "le", "la", "les", "un", "une", "est", "el", "lo",
    "que", "cual", "como", "che", "quale", "come", "il", "o", "os", "as", "qual", "und", "et", "y", "en", "in", "im", "am", "of", "my",
    "me", "i", "ich", "je", "ik", "fuer", "for", "to", "too", "auf", "um", "mit",
];

pub(super) fn classify(blob: &str, pack_weather: bool) -> Option<WeatherClass> {
    let fold = fold_umlaut(blob).replace(['\'', '’', '`'], "");
    let rain = hit(&fold, RAIN);
    let umbrella = umbrella_hit(&fold, rain, pack_weather);
    let weather = weather_topic(&fold, pack_weather);
    let part = part_hint(&fold);
    let day = day_hint(&fold);
    if umbrella {
        return Some(WeatherClass { ask: WeatherAsk::Umbrella, day, part });
    }
    if rain {
        return Some(WeatherClass { ask: WeatherAsk::Rain, day, part });
    }
    if !weather {
        return None;
    }
    let ask = match day {
        DayHint::Tomorrow => WeatherAsk::Tomorrow,
        DayHint::Today | DayHint::Weekend | DayHint::Weekday(_) => WeatherAsk::Today,
        DayHint::Current if hit(&fold, FUTURE) || hit(&fold, FORECAST) => WeatherAsk::Today,
        DayHint::Current => WeatherAsk::Now,
    };
    Some(WeatherClass { ask, day, part })
}

fn weather_topic(fold: &str, pack_weather: bool) -> bool {
    if pack_weather {
        return true;
    }
    if distinctive_hit(fold, WEATHER) || distinctive_hit(fold, FORECAST) {
        return true;
    }
    (hit(fold, WEATHER) || hit(fold, FORECAST)) && leftover_is_weather_ask(fold)
}

fn distinctive_hit(fold: &str, words: &[&str]) -> bool {
    words.iter().any(|word| {
        let needle = fold_umlaut(word);
        if needle.is_empty() {
            return false;
        }
        let distinctive = !needle.is_ascii() || needle.contains(|ch: char| ch.is_whitespace() || ch == '-');
        if !distinctive {
            return false;
        }
        if needle.contains(|ch: char| ch.is_whitespace() || ch == '-') {
            return bounded_contains(fold, &needle);
        }
        fold.contains(&needle)
    })
}

fn leftover_is_weather_ask(fold: &str) -> bool {
    let stripped = strip_morning_words(fold);
    let mut any = false;
    for token in tokens(&stripped) {
        any = true;
        if !weather_function_token(token) {
            return false;
        }
    }
    any
}

fn weather_function_token(token: &str) -> bool {
    ascii_in(token, QUERY_STOP)
        || ascii_in(token, WEATHER)
        || ascii_in(token, FORECAST)
        || ascii_in(token, TODAY)
        || ascii_in(token, TOMORROW)
        || ascii_in(token, FUTURE)
        || ascii_in(token, WEEKEND)
        || ascii_in(token, MORNING)
        || ascii_in(token, AFTERNOON)
        || ascii_in(token, EVENING)
        || ascii_in(token, NIGHT)
        || DAYS.iter().any(|(word, _)| fold_umlaut(word) == token)
}

fn ascii_in(token: &str, words: &[&str]) -> bool {
    words.iter().any(|word| {
        let needle = fold_umlaut(word);
        needle == token && needle.is_ascii() && !needle.contains(|ch: char| ch.is_whitespace() || ch == '-')
    })
}

fn day_hint(fold: &str) -> DayHint {
    let day_fold = strip_morning_words(fold);
    if let Some(wd) = weekday(&day_fold) {
        return DayHint::Weekday(wd);
    }
    if hit(&day_fold, WEEKEND) {
        return DayHint::Weekend;
    }
    if hit(&day_fold, TOMORROW) {
        return DayHint::Tomorrow;
    }
    if hit(fold, TODAY) {
        return DayHint::Today;
    }
    DayHint::Current
}

fn part_hint(fold: &str) -> Option<DayPart> {
    if fold.contains("heute morgen") || fold.contains("this morning") || hit(fold, MORNING) {
        return Some(DayPart::Morning);
    }
    if hit(fold, AFTERNOON) {
        return Some(DayPart::Afternoon);
    }
    if hit(fold, EVENING) {
        return Some(DayPart::Evening);
    }
    if hit(fold, NIGHT) {
        return Some(DayPart::Night);
    }
    None
}

fn umbrella_hit(fold: &str, rain: bool, pack_weather: bool) -> bool {
    if hit(fold, UMBRELLA) {
        return true;
    }
    if !hit(fold, AMBIG_UMBRELLA) {
        return false;
    }
    rain || pack_weather || hit(fold, WEATHER) || hit(fold, NEED) || hit(fold, TODAY) || hit(&strip_morning_words(fold), TOMORROW)
}

fn weekday(fold: &str) -> Option<u8> {
    DAYS.iter().find(|(word, _)| hit(fold, &[word])).map(|(_, day)| *day)
}

fn hit(fold: &str, words: &[&str]) -> bool {
    words.iter().any(|word| {
        let needle = fold_umlaut(word);
        if needle.is_empty() {
            return false;
        }
        if needle.contains(|ch: char| ch.is_whitespace() || ch == '-') {
            return bounded_contains(fold, &needle);
        }
        if needle.is_ascii() {
            tokens(fold).any(|token| token == needle)
        } else {
            fold.contains(&needle)
        }
    })
}

fn bounded_contains(hay: &str, needle: &str) -> bool {
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let start = from + rel;
        let end = start + needle.len();
        let before = start == 0 || !hay[..start].chars().next_back().is_some_and(char::is_alphanumeric);
        let after = end == hay.len() || !hay[end..].chars().next().is_some_and(char::is_alphanumeric);
        if before && after {
            return true;
        }
        from = end;
    }
    false
}

fn tokens(fold: &str) -> impl Iterator<Item = &str> {
    fold.split(|ch: char| !ch.is_alphanumeric()).filter(|part| !part.is_empty())
}

fn strip_morning_words(fold: &str) -> String {
    let mut out = fold.to_string();
    for phrase in GREET_STRIP {
        out = out.replace(&fold_umlaut(phrase), " ");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ask(text: &str, pack: bool) -> Option<WeatherAsk> {
        classify(text, pack).map(|row| row.ask)
    }

    #[test]
    fn rain_and_umbrella_alone() {
        assert_eq!(ask("wird es heute regnen", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("brauche ich heute einen regenschirm", false), Some(WeatherAsk::Umbrella));
        assert_eq!(ask("brauche ich einen schirm", false), Some(WeatherAsk::Umbrella));
        assert_eq!(ask("will it rain today", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("do i need an umbrella", false), Some(WeatherAsk::Umbrella));
        assert_eq!(ask("va t il pleuvoir", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("llueve hoy", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("下雨吗", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("雨が降る", false), Some(WeatherAsk::Rain));
        assert_eq!(ask("сегодня будет дождь", false), Some(WeatherAsk::Rain));
    }

    #[test]
    fn pack_weather_keeps_day() {
        assert_eq!(ask("wie ist das wetter", true), Some(WeatherAsk::Now));
        assert_eq!(ask("wie wird das wetter", true), Some(WeatherAsk::Today));
        assert_eq!(ask("wie wird das wetter heute", true), Some(WeatherAsk::Today));
        assert_eq!(ask("wie wird das wetter morgen", true), Some(WeatherAsk::Tomorrow));
        assert_eq!(ask("what's the weather tomorrow", true), Some(WeatherAsk::Tomorrow));
        assert_eq!(ask("guten morgen wie ist das wetter", true), Some(WeatherAsk::Now));
        assert_eq!(ask("guten morgen wie wird das wetter morgen", true), Some(WeatherAsk::Tomorrow));
        let monday = classify("wie wird das wetter am montag", true).expect("monday");
        assert_eq!(monday.ask, WeatherAsk::Today);
        assert_eq!(monday.day, DayHint::Weekday(0));
        let rain_tmr = classify("wird es morgen regnen", false).expect("rain tomorrow");
        assert_eq!(rain_tmr.ask, WeatherAsk::Rain);
        assert_eq!(rain_tmr.day, DayHint::Tomorrow);
        let afternoon = classify("wird es heute nachmittag regnen", false).expect("afternoon");
        assert_eq!(afternoon.part, Some(DayPart::Afternoon));
        assert_eq!(afternoon.day, DayHint::Today);
    }

    #[test]
    fn calendar_and_bare_day_are_not_weather() {
        assert_eq!(ask("was steht morgen im kalender", false), None);
        assert_eq!(ask("what's on my calendar tomorrow", false), None);
        assert_eq!(ask("heute", false), None);
        assert_eq!(ask("team training at 3", false), None);
        assert_eq!(ask("lampenschirm an", false), None);
        assert_eq!(ask("parasol an", false), None);
        assert_eq!(ask("het weer", false), Some(WeatherAsk::Now));
        assert_eq!(ask("weer", false), None);
        assert_eq!(ask("ma", false), None);
        assert_eq!(ask("weather france", false), None);
        assert_eq!(ask("weather france", true), Some(WeatherAsk::Now));
        assert_eq!(ask("add milk to the shopping list", false), None);
        assert_eq!(ask("weather", false), Some(WeatherAsk::Now));
        assert_eq!(ask("what's the weather", false), Some(WeatherAsk::Now));
        assert_eq!(ask("apaga todo temporizador", false), None);
        assert_eq!(ask("desliga tudo temporizador", false), None);
        assert_eq!(ask("como esta o tempo", false), Some(WeatherAsk::Now));
        assert_eq!(ask("lukk lukk esik", false), None);
        assert_eq!(ask("vali esik", false), None);
    }

    #[test]
    fn rain_and_umbrella_cover_pack_languages() {
        let rows = [
            ("regent het vandaag", WeatherAsk::Rain),
            ("llueve hoy", WeatherAsk::Rain),
            ("piove oggi", WeatherAsk::Rain),
            ("chove hoje", WeatherAsk::Rain),
            ("czy bedzie deszcz", WeatherAsk::Rain),
            ("будет дождь", WeatherAsk::Rain),
            ("下雨吗", WeatherAsk::Rain),
            ("雨が降る", WeatherAsk::Rain),
            ("비가 와", WeatherAsk::Rain),
            ("هل ستمطر", WeatherAsk::Rain),
            ("क्या बारिश होगी", WeatherAsk::Rain),
            ("yağmur yağacak mı", WeatherAsk::Rain),
            ("heb ik een paraplu nodig", WeatherAsk::Umbrella),
            ("necesito un paraguas", WeatherAsk::Umbrella),
            ("preciso de guarda chuva", WeatherAsk::Umbrella),
            ("傘がいる", WeatherAsk::Umbrella),
            ("우산 필요", WeatherAsk::Umbrella),
            ("छाता चाहिए", WeatherAsk::Umbrella),
        ];
        for (text, want) in rows {
            assert_eq!(ask(text, false), Some(want), "{text}");
        }
    }
}
