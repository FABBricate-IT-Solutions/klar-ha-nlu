//! Media commands: naming a player, choosing it by room or by the satellite you're talking
//! to, volume and mute, in English and German. "Music" only means players marked as music
//! players (Music Assistant, Alexa, or a "music" label), never a TV.
use std::collections::BTreeSet;

mod common;
use common::home::{area, describe, effects, ent, home, Talk};
use klar_nlu::types::{EntityRec, HomeGraph, ParseDecision};

fn music(entity: EntityRec) -> EntityRec {
    EntityRec { tags: vec!["music".into()], ..entity }
}

fn test_home(lang: &str, labelled: bool) -> HomeGraph {
    let de = lang == "de";
    let name = |en: &'static str, german: &'static str| if de { german } else { en };
    let speaker = |id: &str, n: &str, a: &str| if labelled { music(ent(id, n, a)) } else { ent(id, n, a) };
    home(
        vec![
            area("kitchen", name("Kitchen", "Küche")),
            area("bedroom", name("Bedroom", "Schlafzimmer")),
            area("living_room", name("Living Room", "Wohnzimmer")),
        ],
        vec![
            speaker("media_player.kitchen_speaker", name("Kitchen Speaker", "Küche Lautsprecher"), "kitchen"),
            speaker("media_player.bedroom_speaker", name("Bedroom Speaker", "Schlafzimmer Lautsprecher"), "bedroom"),
            ent("media_player.living_room_tv", name("Living Room TV", "Wohnzimmer Fernseher"), "living_room"),
        ],
        &[],
    )
}

fn check(lang: &str, labelled: bool, satellite: Option<&str>, text: &str, wanted: &[&str]) {
    let home = test_home(lang, labelled);
    let mut talk = Talk::new(lang, &home).without_confirm();
    talk.session.preferred_area = satellite.map(Into::into);
    let out = talk.say(text);
    let wanted: BTreeSet<String> = wanted.iter().map(|s| s.to_string()).collect();
    assert_eq!(effects(&out, &home), wanted, "[{lang}] {text}: {}", describe(&out));
    if !wanted.is_empty() {
        assert_eq!(out.decision, ParseDecision::Execute, "[{lang}] {text}: {}", describe(&out));
    }
}

#[test]
fn naming_the_player_works_for_any_player() {
    for (lang, text, wanted) in [
        ("en", "pause the kitchen speaker", "HassMediaPause media_player.kitchen_speaker"),
        ("en", "mute the living room tv", "HassMediaPlayerMute media_player.living_room_tv"),
        ("de", "Pausiere den Lautsprecher in der Küche", "HassMediaPause media_player.kitchen_speaker"),
    ] {
        check(lang, false, None, text, &[wanted]);
    }
}

#[test]
fn volume_by_room() {
    for (lang, text, wanted) in [
        ("en", "turn the volume down in the bedroom", "HassSetVolumeRelative media_player.bedroom_speaker volume_step=down"),
        ("en", "set the volume to 30 in the kitchen", "HassSetVolume media_player.kitchen_speaker volume_level=30"),
        ("de", "Stell die Lautstärke in der Küche auf 30", "HassSetVolume media_player.kitchen_speaker volume_level=30"),
    ] {
        check(lang, false, None, text, &[wanted]);
    }
}

#[test]
fn music_by_room_or_satellite_uses_music_players() {
    for (lang, satellite, text, wanted) in [
        ("en", None, "pause the music in the kitchen", "HassMediaPause media_player.kitchen_speaker"),
        ("en", None, "next track in the bedroom", "HassMediaNext media_player.bedroom_speaker"),
        ("en", Some("kitchen"), "pause the music", "HassMediaPause media_player.kitchen_speaker"),
        ("de", None, "Nächstes Lied im Schlafzimmer", "HassMediaNext media_player.bedroom_speaker"),
        ("de", Some("kitchen"), "Pausiere die Musik", "HassMediaPause media_player.kitchen_speaker"),
    ] {
        check(lang, true, satellite, text, &[wanted]);
    }
}

#[test]
fn music_never_means_an_unlabelled_speaker_or_a_tv() {
    check("en", false, None, "pause the music in the kitchen", &[]);
    check("en", true, None, "pause the music in the living room", &[]);
}
