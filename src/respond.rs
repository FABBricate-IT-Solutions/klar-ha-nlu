use crate::lang::{speech_lang, LangId};
use crate::normalize::compact;
use crate::types::{HomeGraph, Intent, Personality};

pub fn speak(intents: &[Intent], personality: Personality, clarify: bool, home: Option<&HomeGraph>) -> String {
    let en = speech_lang() == LangId::En;
    if clarify {
        return if en { "Tell me which device you mean.".into() } else { "Sag mir welches Gerät du meinst.".into() };
    }
    if intents.is_empty() {
        return speak_unknown();
    }
    wrap(personality, &describe_all(intents, home, en), en)
}

pub fn speak_unknown() -> String {
    if speech_lang() == LangId::En {
        "I did not match that. Try: turn on the living room light.".into()
    } else {
        "Das habe ich nicht zugeordnet. Sag zum Beispiel: Licht im Wohnzimmer an.".into()
    }
}

pub fn speak_need_target(off: bool) -> String {
    if speech_lang() == LangId::En {
        if off {
            "What should I turn off?".into()
        } else {
            "What should I turn on?".into()
        }
    } else if off {
        "Was soll ich ausmachen?".into()
    } else {
        "Was soll ich einschalten?".into()
    }
}

pub fn speak_correction() -> String {
    if speech_lang() == LangId::En {
        "Noted. I will treat the last sentence as a misread.".into()
    } else {
        "Notiert. Den letzten Satz lege ich als Fehlinterpretation ab.".into()
    }
}

pub fn speak_clarify(names: &[String]) -> String {
    let labels: Vec<String> = names.iter().map(|id| id.rsplit('.').next().unwrap_or(id).replace('_', " ")).collect();
    if speech_lang() == LangId::En {
        format!("Do you mean {}?", labels.join(" or "))
    } else {
        format!("Meinst du {}?", labels.join(" oder "))
    }
}

fn describe_all(intents: &[Intent], home: Option<&HomeGraph>, en: bool) -> String {
    if intents.len() > 1 && intents.iter().all(|i| i.name == intents[0].name) {
        let names: Vec<String> = intents.iter().map(|i| spoken_where(i, home, en)).filter(|s| !s.is_empty()).collect();
        if names.len() > 1 {
            if let Some(group) = describe_group(&intents[0].name, &names, en) {
                return group;
            }
        }
    }
    intents.iter().map(|i| describe(i, home, en)).collect::<Vec<_>>().join(" ")
}

fn describe_group(name: &str, wheres: &[String], en: bool) -> Option<String> {
    let joined = join_names(wheres, en);
    Some(match (name, en) {
        ("HassTurnOn", false) => format!("{joined} sind an."),
        ("HassTurnOn", true) => format!("{joined} are on."),
        ("HassTurnOff", false) => format!("{joined} sind aus."),
        ("HassTurnOff", true) => format!("{joined} are off."),
        _ => return None,
    })
}

fn join_names(names: &[String], en: bool) -> String {
    let conj = if en { " and " } else { " und " };
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [.., last] => format!("{}{conj}{last}", names[..names.len() - 1].join(", ")),
    }
}

fn describe(intent: &Intent, home: Option<&HomeGraph>, en: bool) -> String {
    let where_ = spoken_where(intent, home, en);
    let area = intent.slot("area").unwrap_or("");
    if en {
        describe_en(intent, &where_, area)
    } else {
        describe_de(intent, &where_, area)
    }
}

fn spoken_where(intent: &Intent, home: Option<&HomeGraph>, en: bool) -> String {
    if let Some(id) = intent.slot("entity_id") {
        if looks_started(id) {
            return scene_label(id, home);
        }
        if let Some(ent) = home.and_then(|h| h.entities.iter().find(|e| e.entity_id == id)) {
            return device_label(&ent.name, &ent.domain, en);
        }
        return device_label(&object_id(id), domain_of(id), en);
    }
    if let Some(area) = intent.slot("area") {
        return area_label(area, intent.slot("domain").unwrap_or(""), &intent.name, en);
    }
    String::new()
}

fn looks_started(id: &str) -> bool {
    id.starts_with("scene.") || id.starts_with("script.")
}

fn scene_label(id: &str, home: Option<&HomeGraph>) -> String {
    home.and_then(|h| h.entities.iter().find(|e| e.entity_id == id))
        .map(|e| pretty_device(&e.name))
        .unwrap_or_else(|| pretty_device(&object_id(id)))
}

fn device_label(name: &str, domain: &str, en: bool) -> String {
    let pretty = pretty_device(name);
    let folded = compact(&pretty);
    let named = folded.contains("licht")
        || folded.contains("lampe")
        || folded.contains("leuchte")
        || folded.contains("light")
        || folded.contains("lamp")
        || folded.contains("kugel");
    if domain == "light" && !named {
        if en {
            format!("{pretty} light")
        } else {
            format!("{pretty}licht")
        }
    } else {
        pretty
    }
}

fn area_label(area: &str, domain: &str, intent: &str, en: bool) -> String {
    let light = domain == "light" || matches!(intent, "HassTurnOn" | "HassTurnOff" | "HassToggle" | "HassLightSet");
    if light && domain != "climate" && domain != "fan" && domain != "media_player" && domain != "switch" {
        if en {
            format!("the light {}", loc(area, true))
        } else {
            format!("Licht {}", loc(area, false))
        }
    } else {
        title_word(area)
    }
}

fn object_id(id: &str) -> String {
    id.rsplit('.').next().unwrap_or(id).replace('_', " ")
}

fn domain_of(id: &str) -> &str {
    id.split('.').next().unwrap_or("")
}

fn describe_de(intent: &Intent, where_: &str, area: &str) -> String {
    let target = or_home(where_, false);
    match intent.name.as_str() {
        "HassTurnOn" if looks_started(intent.slot("entity_id").unwrap_or("")) => format!("{target} ist gestartet."),
        "HassTurnOn" => format!("{target} ist an."),
        "HassTurnOff" => format!("{target} ist aus."),
        "HassToggle" => format!("{target} ist umgeschaltet."),
        "HassLightSet" => {
            let bri = intent.slot("brightness").unwrap_or("?");
            format!("{target} auf {bri} Prozent.")
        }
        "HassClimateSetTemperature" => {
            let t = intent.slot("temperature").unwrap_or("?");
            format!("Heizung {target} auf {t} Grad.")
        }
        "HassGetState" => {
            if intent.slot("device_class") == Some("temperature") {
                format!("Temperatur {}.", loc(area, false))
            } else {
                format!("Ich prüfe {target}.")
            }
        }
        "HassMediaPause" => "Wiedergabe ist pausiert.".into(),
        "HassMediaUnpause" => "Wiedergabe läuft weiter.".into(),
        "HassMediaNext" => "Nächster Titel.".into(),
        "HassMediaPlayerMute" => "Ton ist aus.".into(),
        "HassFanSetSpeed" => {
            let p = intent.slot("percentage").unwrap_or("?");
            format!("Lüfter auf {p} Prozent.")
        }
        "HassVacuumStart" => format!("{} saugt jetzt.", vacuum_name(where_, false)),
        "HassVacuumReturnToBase" => format!("{} fährt zur Station.", vacuum_name(where_, false)),
        "HassStartTimer" => "Timer läuft.".into(),
        "HassCancelTimer" => "Timer ist aus.".into(),
        "HassPauseTimer" => "Timer ist pausiert.".into(),
        "HassListAddItem" | "HassShoppingListAddItem" => "Steht auf der Liste.".into(),
        other => format!("Erledigt: {other}."),
    }
}

fn describe_en(intent: &Intent, where_: &str, area: &str) -> String {
    let target = or_home(where_, true);
    match intent.name.as_str() {
        "HassTurnOn" if looks_started(intent.slot("entity_id").unwrap_or("")) => format!("Started {target}."),
        "HassTurnOn" => format!("{target} is on."),
        "HassTurnOff" => format!("{target} is off."),
        "HassToggle" => format!("{target} is toggled."),
        "HassLightSet" => {
            let bri = intent.slot("brightness").unwrap_or("?");
            format!("{target} is at {bri} percent.")
        }
        "HassClimateSetTemperature" => {
            let t = intent.slot("temperature").unwrap_or("?");
            format!("Heat {target} is at {t} degrees.")
        }
        "HassGetState" => {
            if intent.slot("device_class") == Some("temperature") {
                format!("Temperature {}.", loc(area, true))
            } else {
                format!("Checking {target}.")
            }
        }
        "HassMediaPause" => "Playback is paused.".into(),
        "HassMediaUnpause" => "Playback is back on.".into(),
        "HassMediaNext" => "Next track.".into(),
        "HassMediaPlayerMute" => "Muted.".into(),
        "HassFanSetSpeed" => {
            let p = intent.slot("percentage").unwrap_or("?");
            format!("Fan is at {p} percent.")
        }
        "HassVacuumStart" => format!("{} is vacuuming.", vacuum_name(where_, true)),
        "HassVacuumReturnToBase" => format!("{} is heading to the dock.", vacuum_name(where_, true)),
        "HassStartTimer" => "Timer is running.".into(),
        "HassCancelTimer" => "Timer is off.".into(),
        "HassPauseTimer" => "Timer is paused.".into(),
        "HassListAddItem" | "HassShoppingListAddItem" => "Added to the list.".into(),
        other => format!("Done: {other}."),
    }
}

fn vacuum_name(where_: &str, en: bool) -> String {
    if where_.is_empty() || where_ == "Zuhause" || where_ == "home" {
        if en {
            "The vacuum".into()
        } else {
            "Staubsauger".into()
        }
    } else {
        where_.to_string()
    }
}

fn pretty_device(raw: &str) -> String {
    let parts: Vec<&str> = raw.split_whitespace().filter(|p| !p.is_empty()).collect();
    let tail = parts.last().copied().unwrap_or("");
    let light = matches!(tail, "licht" | "lichter" | "lampe" | "lampen" | "light" | "lights");
    let all = parts.first().is_some_and(|p| matches!(*p, "alle" | "all" | "every"));
    if light && !all && parts.len() >= 2 {
        let head = parts[..parts.len() - 1].join("");
        return title_word(&format!("{head}{tail}"));
    }
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i > 0 && matches!(*part, "und" | "and" | "im" | "in" | "the" | "der" | "die" | "das") {
                (*part).to_string()
            } else {
                title_word(part)
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn title_word(raw: &str) -> String {
    let mut chars = raw.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

fn loc(area: &str, en: bool) -> String {
    if area.is_empty() {
        return if en { "in the home".into() } else { "in der Wohnung".into() };
    }
    let folded = compact(area);
    let room = match folded.as_str() {
        "kuche" | "kueche" => if en { "kitchen" } else { "Küche" }.to_string(),
        "wohnung" => if en { "home" } else { "Wohnung" }.to_string(),
        _ => title_word(&area.replace('_', " ")),
    };
    if en {
        format!("in the {room}")
    } else if matches!(folded.as_str(), "kuche" | "kueche" | "wohnung") {
        format!("in der {room}")
    } else {
        format!("im {room}")
    }
}

fn or_home(s: &str, en: bool) -> String {
    if s.is_empty() {
        if en {
            "the device".into()
        } else {
            "das Gerät".into()
        }
    } else {
        s.to_string()
    }
}

fn wrap(personality: Personality, body: &str, en: bool) -> String {
    if matches!(personality, Personality::Default) {
        return body.to_string();
    }
    let prefix = match (personality, en) {
        (Personality::Butler, false) => "Sehr wohl. ",
        (Personality::Butler, true) => "Very well. ",
        (Personality::Locker, false) => "Geht klar. ",
        (Personality::Locker, true) => "Got it. ",
        (Personality::Fuersorglich, false) => "Mache ich sofort. ",
        (Personality::Fuersorglich, true) => "Doing that now. ",
        (Personality::Party, false) => "Läuft! ",
        (Personality::Party, true) => "Let's go! ",
        (Personality::Grantig, false) => "Schon gut. ",
        (Personality::Grantig, true) => "Fine. ",
        (Personality::Sarkastisch, false) => "Wie überraschend, wieder ein Befehl. ",
        (Personality::Sarkastisch, true) => "What a surprise, another command. ",
        (Personality::Pirat, false) => "Aye. ",
        (Personality::Pirat, true) => "Aye. ",
        (Personality::Hippie, false) => "Alles easy. ",
        (Personality::Hippie, true) => "All good. ",
        (Personality::Gollum, false) => "Ja, mein Schatz. ",
        (Personality::Gollum, true) => "Yes, my precious. ",
        (Personality::Default, _) => "",
    };
    format!("{prefix}{body}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::bind;
    use crate::types::Intent;

    #[test]
    fn speech_follows_pinned_pack() {
        let intent = Intent::new("HassTurnOn").with("area", "wohnzimmer");
        let _de = bind(&["de".into()]);
        let de = speak(std::slice::from_ref(&intent), Personality::Default, false, None);
        assert!(de.contains("ist an") || de.contains("Licht"), "{de}");
        drop(_de);
        let _en = bind(&["en".into()]);
        let en = speak(&[intent], Personality::Default, false, None);
        assert!(en.contains("is on") || en.contains("light"), "{en}");
        assert!(!en.contains("ist an"), "{en}");
    }

    #[test]
    fn speech_compounds_room_light() {
        let _de = bind(&["de".into()]);
        let intent = Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer_licht");
        let de = speak(&[intent], Personality::Default, false, None);
        assert_eq!(de, "Schlafzimmerlicht ist an.");
        assert!(!de.contains("light."));
        let kugel = Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer");
        assert_eq!(speak(&[kugel], Personality::Default, false, None), "Schlafzimmerlicht ist an.");
        let butler = speak(&[Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer")], Personality::Butler, false, None);
        assert_eq!(butler, "Sehr wohl. Schlafzimmerlicht ist an.");
    }
}
