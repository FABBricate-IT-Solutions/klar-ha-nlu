//! Exact spoken replies for media actions and window lists, so wording changes are deliberate.
use klar_nlu::speech::render_snapshot;
use klar_nlu::types::SpeechSnapshot;
use serde_json::{json, Value};

fn reply(lang: &str, intent: &str, slots: &[(&str, &str)], entities: Value) -> String {
    let slots: Vec<Value> = slots.iter().map(|(name, value)| json!({"name": name, "value": value})).collect();
    let snapshot: SpeechSnapshot = serde_json::from_value(json!({
        "language": lang, "outcome": "success",
        "intent": {"name": intent, "slots": slots},
        "entities": entities
    }))
    .expect("valid speech snapshot");
    render_snapshot(&snapshot).speech
}

fn speaker() -> Value {
    json!([{"entity_id": "media_player.kitchen_speaker", "name": "Kitchen Speaker", "domain": "media_player", "state": "playing"}])
}

#[test]
fn media_action_replies() {
    let named = [("entity_id", "media_player.kitchen_speaker"), ("name", "Kitchen Speaker")];
    let with = |extra: (&'static str, &'static str)| [named[0], named[1], extra];
    for (lang, intent, slots, wanted) in [
        ("en", "HassMediaPause", named.to_vec(), "Kitchen Speaker is paused."),
        ("en", "HassMediaUnpause", named.to_vec(), "Kitchen Speaker resumed playback."),
        ("en", "HassMediaNext", named.to_vec(), "The next track is playing on Kitchen Speaker."),
        ("en", "HassMediaPrevious", named.to_vec(), "The previous track is playing on Kitchen Speaker."),
        ("en", "HassMediaPlayerMute", named.to_vec(), "Kitchen Speaker is muted."),
        ("en", "HassMediaPlayerUnmute", named.to_vec(), "Kitchen Speaker is unmuted."),
        ("en", "HassSetVolume", with(("volume_level", "30")).to_vec(), "Kitchen Speaker volume is set to 30 percent."),
        ("en", "HassSetVolumeRelative", with(("volume_step", "down")).to_vec(), "Kitchen Speaker volume was lowered."),
        ("en", "HassSetVolumeRelative", with(("volume_step", "up")).to_vec(), "Kitchen Speaker volume was raised."),
        ("de", "HassMediaPause", named.to_vec(), "Kitchen Speaker ist pausiert."),
        ("de", "HassMediaNext", named.to_vec(), "Auf Kitchen Speaker läuft der nächste Titel."),
        ("de", "HassMediaPlayerUnmute", named.to_vec(), "Der Ton von Kitchen Speaker ist an."),
        ("de", "HassSetVolume", with(("volume_level", "30")).to_vec(), "Die Lautstärke von Kitchen Speaker ist auf 30 Prozent."),
        ("de", "HassSetVolumeRelative", with(("volume_step", "down")).to_vec(), "Die Lautstärke von Kitchen Speaker wurde verringert."),
    ] {
        assert_eq!(reply(lang, intent, &slots, speaker()), wanted, "[{lang}] {intent} {slots:?}");
    }
}

#[test]
fn play_reply_names_title_and_artist_when_known() {
    let playing = |attributes: Value| {
        json!([{"entity_id": "media_player.kitchen_speaker", "name": "Kitchen Speaker", "domain": "media_player",
                "state": "playing", "attributes": attributes}])
    };
    for (lang, attributes, wanted) in [
        ("en", json!({"media_title": "Blue", "media_artist": "Joni Mitchell"}), "Playing Blue by Joni Mitchell."),
        ("en", json!({"media_title": "Blue"}), "Playing Blue."),
        ("en", json!({}), "Playback started."),
        ("de", json!({"media_title": "Blue", "media_artist": "Joni Mitchell"}), "Ich spiele Blue von Joni Mitchell."),
        ("de", json!({}), "Die Wiedergabe wurde gestartet."),
    ] {
        assert_eq!(reply(lang, "HassMediaSearchAndPlay", &[], playing(attributes.clone())), wanted, "[{lang}] {attributes}");
    }
}

#[test]
fn window_list_replies() {
    let sensors = json!([
        {"entity_id": "binary_sensor.kitchen_window", "name": "Kitchen Window", "domain": "binary_sensor", "state": "on", "device_class": "window"},
        {"entity_id": "binary_sensor.office_window", "name": "Office Window", "domain": "binary_sensor", "state": "off", "device_class": "window"},
    ]);
    fn ask(state: Option<&'static str>) -> Vec<(&'static str, &'static str)> {
        let mut slots = vec![("domain", "binary_sensor"), ("device_class", "window"), ("response", "list")];
        slots.extend(state.map(|s| ("state", s)));
        slots
    }
    for (lang, state, entities, wanted) in [
        ("en", Some("on"), sensors.clone(), "These windows are open: Kitchen Window."),
        ("en", Some("off"), sensors.clone(), "These windows are closed: Office Window."),
        ("en", None, sensors.clone(), "Window sensors: Kitchen Window, Office Window."),
        ("en", Some("on"), json!([]), "No windows are open."),
        ("de", Some("on"), sensors.clone(), "Diese Fenster sind offen: Kitchen Window."),
        ("de", Some("off"), json!([]), "Es sind keine Fenster geschlossen."),
        ("nl", Some("on"), sensors.clone(), "Deze ramen staan open: Kitchen Window."),
        ("nl", None, json!([]), "Er zijn geen raamsensoren gevonden."),
    ] {
        assert_eq!(reply(lang, "HassGetState", &ask(state), entities), wanted, "[{lang}] state={state:?}");
    }
}
