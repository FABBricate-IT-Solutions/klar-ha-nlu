use crate::lang::{speech_lang, LangId};
use crate::types::{Intent, Personality};

pub fn speak(intents: &[Intent], personality: Personality, clarify: bool) -> String {
    let en = speech_lang() == LangId::En;
    if clarify {
        return if en {
            "Tell me which device you mean.".into()
        } else {
            "Sag mir welches Gerät du meinst.".into()
        };
    }
    if intents.is_empty() {
        return speak_unknown();
    }
    let body = intents.iter().map(describe).collect::<Vec<_>>().join(" ");
    wrap(personality, &body, en)
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
    let labels: Vec<String> = names
        .iter()
        .map(|id| id.rsplit('.').next().unwrap_or(id).replace('_', " "))
        .collect();
    if speech_lang() == LangId::En {
        format!("Do you mean {}?", labels.join(" or "))
    } else {
        format!("Meinst du {}?", labels.join(" oder "))
    }
}

fn describe(intent: &Intent) -> String {
    let area = intent.slot("area").unwrap_or("");
    let entity = intent
        .slot("entity_id")
        .map(short_id)
        .unwrap_or_default();
    let where_ = if !entity.is_empty() {
        entity
    } else if !area.is_empty() {
        area.to_string()
    } else {
        String::new()
    };
    if speech_lang() == LangId::En {
        describe_en(intent, &where_, area)
    } else {
        describe_de(intent, &where_, area)
    }
}

fn describe_de(intent: &Intent, where_: &str, area: &str) -> String {
    match intent.name.as_str() {
        "HassTurnOn" => format!("Schalte {} ein.", or_home(where_, false)),
        "HassTurnOff" => format!("Schalte {} aus.", or_home(where_, false)),
        "HassToggle" => format!("Schalte {} um.", or_home(where_, false)),
        "HassLightSet" => {
            let bri = intent.slot("brightness").unwrap_or("?");
            format!("Setze {} auf {bri} Prozent.", or_home(where_, false))
        }
        "HassClimateSetTemperature" => {
            let t = intent.slot("temperature").unwrap_or("?");
            format!("Heizung {} auf {t} Grad.", where_.trim())
        }
        "HassGetState" => {
            if intent.slot("device_class") == Some("temperature") {
                format!("Frage die Temperatur {} ab.", loc(area, false))
            } else {
                format!("Frage den Zustand {} ab.", or_home(where_, false))
            }
        }
        "HassMediaPause" => "Pausiere die Wiedergabe.".into(),
        "HassMediaUnpause" => "Setze die Wiedergabe fort.".into(),
        "HassMediaNext" => "Nächster Titel.".into(),
        "HassMediaPlayerMute" => "Stumm.".into(),
        "HassFanSetSpeed" => {
            let p = intent.slot("percentage").unwrap_or("?");
            format!("Lüfter auf {p} Prozent.")
        }
        "HassVacuumStart" => "R2D2 soll saugen.".into(),
        "HassVacuumReturnToBase" => "R2D2 zurück zur Station.".into(),
        "HassStartTimer" => "Timer gestartet.".into(),
        "HassCancelTimer" => "Timer abgebrochen.".into(),
        "HassPauseTimer" => "Timer pausiert.".into(),
        "HassListAddItem" | "HassShoppingListAddItem" => "Auf die Liste.".into(),
        other => format!("Führe {other} aus."),
    }
}

fn describe_en(intent: &Intent, where_: &str, area: &str) -> String {
    match intent.name.as_str() {
        "HassTurnOn" => format!("Turn on {}.", or_home(where_, true)),
        "HassTurnOff" => format!("Turn off {}.", or_home(where_, true)),
        "HassToggle" => format!("Toggle {}.", or_home(where_, true)),
        "HassLightSet" => {
            let bri = intent.slot("brightness").unwrap_or("?");
            format!("Set {} to {bri} percent.", or_home(where_, true))
        }
        "HassClimateSetTemperature" => {
            let t = intent.slot("temperature").unwrap_or("?");
            format!("Set heat {} to {t} degrees.", where_.trim())
        }
        "HassGetState" => {
            if intent.slot("device_class") == Some("temperature") {
                format!("Checking the temperature {}.", loc(area, true))
            } else {
                format!("Checking the state of {}.", or_home(where_, true))
            }
        }
        "HassMediaPause" => "Pausing playback.".into(),
        "HassMediaUnpause" => "Resuming playback.".into(),
        "HassMediaNext" => "Next track.".into(),
        "HassMediaPlayerMute" => "Muted.".into(),
        "HassFanSetSpeed" => {
            let p = intent.slot("percentage").unwrap_or("?");
            format!("Fan to {p} percent.")
        }
        "HassVacuumStart" => "R2D2 will vacuum.".into(),
        "HassVacuumReturnToBase" => "R2D2 is returning to the dock.".into(),
        "HassStartTimer" => "Timer started.".into(),
        "HassCancelTimer" => "Timer cancelled.".into(),
        "HassPauseTimer" => "Timer paused.".into(),
        "HassListAddItem" | "HassShoppingListAddItem" => "Added to the list.".into(),
        other => format!("Running {other}."),
    }
}

fn short_id(id: &str) -> String {
    pretty_device(&id.rsplit('.').next().unwrap_or(id).replace('_', " "))
}

fn pretty_device(raw: &str) -> String {
    let parts: Vec<&str> = raw.split_whitespace().filter(|p| !p.is_empty()).collect();
    let tail = parts.last().copied().unwrap_or("");
    let light = matches!(tail, "licht" | "lichter" | "lampe" | "lampen" | "light" | "lights");
    let all = parts.first().is_some_and(|p| matches!(*p, "alle" | "all" | "every"));
    if light && !all && parts.len() >= 2 {
        let head = parts[..parts.len() - 1].join("");
        let mut out = format!("{head}{tail}");
        if let Some(first) = out.chars().next() {
            out.replace_range(..first.len_utf8(), &first.to_uppercase().to_string());
        }
        return out;
    }
    raw.to_string()
}

fn loc(area: &str, en: bool) -> String {
    if area.is_empty() {
        if en {
            "in the home".into()
        } else {
            "in der Wohnung".into()
        }
    } else if en {
        format!("in the {area}")
    } else {
        format!("im {area}")
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
        let de = speak(&[intent.clone()], Personality::Default, false);
        assert!(de.contains("Schalte"), "{de}");
        drop(_de);
        let _en = bind(&["en".into()]);
        let en = speak(&[intent], Personality::Default, false);
        assert!(en.contains("Turn on"), "{en}");
        assert!(!en.contains("Schalte"), "{en}");
    }

    #[test]
    fn speech_compounds_room_light() {
        let _de = bind(&["de".into()]);
        let intent = Intent::new("HassTurnOn").with("entity_id", "light.schlafzimmer_licht");
        let de = speak(&[intent], Personality::Default, false);
        assert_eq!(de, "Schalte Schlafzimmerlicht ein.");
        assert!(!de.contains(','));
    }
}
