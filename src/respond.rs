use crate::types::{Intent, Personality};

pub fn speak(intents: &[Intent], personality: Personality, clarify: bool) -> String {
    if clarify {
        return "Sag mir welches Gerät du meinst.".into();
    }
    if intents.is_empty() {
        return "Nichts erkannt.".into();
    }
    let body = intents.iter().map(describe).collect::<Vec<_>>().join(" ");
    wrap(personality, &body)
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

    match intent.name.as_str() {
        "HassTurnOn" => format!("Schalte {} ein.", or_home(&where_)),
        "HassTurnOff" => format!("Schalte {} aus.", or_home(&where_)),
        "HassToggle" => format!("Schalte {} um.", or_home(&where_)),
        "HassLightSet" => {
            let bri = intent.slot("brightness").unwrap_or("?");
            format!("Setze {} auf {bri} Prozent.", or_home(&where_))
        }
        "HassClimateSetTemperature" => {
            let t = intent.slot("temperature").unwrap_or("?");
            format!("Heizung {} auf {t} Grad.", where_.trim())
        }
        "HassGetState" => {
            if intent.slot("device_class") == Some("temperature") {
                format!("Frage die Temperatur {} ab.", loc(area))
            } else {
                format!("Frage den Zustand {} ab.", or_home(&where_))
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
        other => format!("Führe {other} aus."),
    }
}

fn short_id(id: &str) -> String {
    id.rsplit('.')
        .next()
        .unwrap_or(id)
        .replace('_', " ")
}

fn loc(area: &str) -> String {
    if area.is_empty() {
        "in der Wohnung".into()
    } else {
        format!("im {area}")
    }
}

fn or_home(s: &str) -> String {
    if s.is_empty() {
        "das Gerät".into()
    } else {
        s.to_string()
    }
}

fn wrap(personality: Personality, body: &str) -> String {
    match personality {
        Personality::Default => body.to_string(),
        Personality::Butler => format!("Sehr wohl. {body}"),
        Personality::Locker => format!("Geht klar. {body}"),
        Personality::Fuersorglich => format!("Mache ich sofort. {body}"),
        Personality::Party => format!("Läuft! {body}"),
        Personality::Grantig => format!("Schon gut. {body}"),
        Personality::Sarkastisch => format!("Wie überraschend, wieder ein Befehl. {body}"),
        Personality::Pirat => format!("Aye. {body}"),
        Personality::Hippie => format!("Alles easy. {body}"),
        Personality::Gollum => format!("Ja, mein Schatz. {body}"),
    }
}
