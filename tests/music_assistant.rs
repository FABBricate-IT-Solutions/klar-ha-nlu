use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
use klar_nlu::parse::split::split_clauses;
use klar_nlu::session::Session;
use klar_nlu::types::{EntityRec, HomeGraph, Intent, Settings};

fn settings(lang: &str) -> Settings {
    Settings { languages: vec![lang.into()], ..Settings::default() }
}

fn entity(id: &str, name: &str, domain: &str, platform: Option<&str>, area: Option<&str>, aliases: &[&str], tags: &[&str]) -> EntityRec {
    EntityRec {
        entity_id: id.into(),
        name: name.into(),
        domain: domain.into(),
        platform: platform.map(str::to_string),
        area: area.map(str::to_string),
        aliases: aliases.iter().map(|alias| (*alias).to_string()).collect(),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
    }
}

fn home() -> HomeGraph {
    let mut home = default_home();
    home.entities.push(entity(
        "media_player.wohnzimmer_2",
        "Wohnzimmer Soundbar",
        "media_player",
        Some("music_assistant"),
        Some("wohnzimmer"),
        &["soundbar", "wohnzimmer musik"],
        &["Musik"],
    ));
    home.entities.push(entity(
        "media_player.wohnzimmer_tv",
        "Wohnzimmer TV",
        "media_player",
        None,
        Some("wohnzimmer"),
        &["tv", "fernseher"],
        &[],
    ));
    home.entities.push(entity(
        "script.llm_script_for_music_assistant_voice_requests_3",
        "musik",
        "script",
        None,
        Some("wohnzimmer"),
        &["music assistant", "musik", "radio"],
        &[],
    ));
    home
}

fn home_with_kitchen_player() -> HomeGraph {
    let mut home = home();
    home.entities.push(entity(
        "media_player.kuche_2",
        "Küche Lautsprecher",
        "media_player",
        Some("music_assistant"),
        Some("kuche"),
        &["kueche musik", "kitchen speaker"],
        &["Musik"],
    ));
    home
}

fn one(text: &str, lang: &str) -> Intent {
    let home = home();
    let mut session = Session::new();
    let result = parse(text, &home, &mut session, &[], &settings(lang));
    assert_eq!(result.intents.len(), 1, "{text}: {:?}", result.intents);
    result.intents.into_iter().next().unwrap()
}

fn one_with_area(text: &str, lang: &str, preferred_area: &str, home: HomeGraph) -> Intent {
    let mut session = Session::new();
    session.preferred_area = Some(preferred_area.into());
    let result = parse(text, &home, &mut session, &[], &settings(lang));
    assert_eq!(result.intents.len(), 1, "{text}: {:?}", result.intents);
    result.intents.into_iter().next().unwrap()
}

fn intents(text: &str, lang: &str, home: &HomeGraph, session: &mut Session) -> Vec<Intent> {
    parse(text, home, session, &[], &settings(lang)).intents
}

fn add_living_room_player(home: &mut HomeGraph, id: &str, tags: &[&str]) {
    home.entities.push(entity(id, "Wohnzimmer Box", "media_player", Some("music_assistant"), Some("wohnzimmer"), &["zweite box"], tags));
}

fn slot<'a>(intent: &'a Intent, name: &str) -> Option<&'a str> {
    intent.slot(name)
}

#[test]
fn de_simple_play_uses_mass_play_on_ma_player() {
    let intent = one("Spiel Queen", "de");
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "media_id"), Some("queen"));
    assert_eq!(slot(&intent, "search_query"), Some("queen"));
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn de_play_music_in_area_unpauses_instead_of_turning_on() {
    let intent = one("Spiele Musik im Wohnzimmer", "de");
    assert_eq!(intent.name, "HassMediaUnpause");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
    assert_ne!(intent.name, "HassTurnOn");
}

#[test]
fn de_play_artist_in_area_uses_mass() {
    let intent = one("Spiele Linkin Park im Wohnzimmer", "de");
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "media_id"), Some("linkin park"));
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn de_album_by_artist_uses_mass_play_media() {
    let intent = one("Spiel das Album Rumours von Fleetwood Mac", "de");
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "media_id"), Some("rumours"));
    assert_eq!(slot(&intent, "media_type"), Some("album"));
    assert_eq!(slot(&intent, "artist"), Some("fleetwood mac"));
}

#[test]
fn en_playlist_in_area_keeps_media_type() {
    let intent = one("Play the playlist Chill in the living room", "en");
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "media_id"), Some("chill"));
    assert_eq!(slot(&intent, "media_class"), Some("playlist"));
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn radio_mode_uses_mass_play_media() {
    let intent = one("Play Queen using radio mode", "en");
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "media_id"), Some("queen"));
    assert_eq!(slot(&intent, "radio_mode"), Some("true"));
}

#[test]
fn musik_an_resumes_ma_player_not_script_alias() {
    let intent = one("Musik an", "de");
    assert_eq!(intent.name, "HassMediaUnpause");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn kitchen_alexa_player_is_music_target() {
    let mut home = default_home();
    home.entities.push(entity(
        "media_player.kuchenbereich_2",
        "Küchenbereich",
        "media_player",
        Some("alexa_devices"),
        Some("kuche"),
        &["kueche", "kitchen"],
        &["assist"],
    ));
    let mut session = Session::new();
    let result = parse("Musik in der Küche", &home, &mut session, &[], &settings("de"));
    assert!(!result.clarify, "{result:?}");
    assert_eq!(result.intents.first().map(|intent| intent.name.as_str()), Some("HassMediaUnpause"), "{result:?}");
    assert_eq!(result.intents.first().and_then(|intent| intent.slot("entity_id")), Some("media_player.kuchenbereich_2"), "{result:?}");
}

#[test]
fn tv_an_stays_plain_media_player_turn_on() {
    let intent = one("Wohnzimmer TV an", "de");
    assert_eq!(intent.name, "HassTurnOn");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_tv"));
}

#[test]
fn volume_and_previous_target_music_assistant() {
    let volume = one("Lautstärke 30", "de");
    assert_eq!(volume.name, "HassSetVolume");
    assert_eq!(slot(&volume, "volume_level"), Some("30"));
    assert_eq!(slot(&volume, "entity_id"), Some("media_player.wohnzimmer_2"));

    let previous = one("Vorheriges Lied", "de");
    assert_eq!(previous.name, "HassMediaPrevious");
    assert_eq!(slot(&previous, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn queue_transfer_and_favorite_are_mass_intents() {
    let queue = one("Queue Bohemian Rhapsody", "en");
    assert_eq!(queue.name, "MassPlayMedia");
    assert_eq!(slot(&queue, "media_id"), Some("bohemian rhapsody"));
    assert_eq!(slot(&queue, "enqueue"), Some("add"));

    let home = home_with_kitchen_player();
    let mut session = Session::new();
    session.remember_entity("media_player.kuche_2");
    let mut parsed = intents("Move music to the living room", "en", &home, &mut session);
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    let transfer = parsed.remove(0);
    assert_eq!(transfer.name, "MassTransferQueue");
    assert_eq!(slot(&transfer, "entity_id"), Some("media_player.wohnzimmer_2"));
    assert_eq!(slot(&transfer, "source_player"), Some("media_player.kuche_2"));

    let favorite = one("Favorisiere den Titel", "de");
    assert_eq!(favorite.name, "MassFavorite");
    assert_eq!(slot(&favorite, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn status_queries_do_not_start_playback() {
    let now = one("Was läuft", "de");
    assert_eq!(now.name, "HassGetState");
    assert_eq!(slot(&now, "media_status"), Some("now_playing"));
    assert_eq!(slot(&now, "entity_id"), Some("media_player.wohnzimmer_2"));

    let volume = one("Wie laut ist die Musik", "de");
    assert_eq!(volume.name, "HassGetState");
    assert_eq!(slot(&volume, "media_status"), Some("volume"));
    assert_eq!(slot(&volume, "entity_id"), Some("media_player.wohnzimmer_2"));

    let queue = one("Was kommt als nächstes Lied", "de");
    assert_eq!(queue.name, "MassGetQueue");
    assert_eq!(slot(&queue, "media_status"), Some("queue"));
    assert_eq!(slot(&queue, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn english_status_queries_target_area_player() {
    let now = one("What's playing in the living room", "en");
    assert_eq!(now.name, "HassGetState");
    assert_eq!(slot(&now, "media_status"), Some("now_playing"));
    assert_eq!(slot(&now, "entity_id"), Some("media_player.wohnzimmer_2"));

    let queue = one("What's next in the queue", "en");
    assert_eq!(queue.name, "MassGetQueue");
    assert_eq!(slot(&queue, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn climate_like_question_is_not_media_favorite_or_status() {
    let intent = one("What is the climate like in the living room", "en");
    assert_eq!(intent.name, "HassGetState");
    assert_ne!(slot(&intent, "media_status"), Some("player"));
}

#[test]
fn preferred_area_selects_local_music_assistant_player() {
    let living = one_with_area("Spiel Queen", "de", "wohnzimmer", home_with_kitchen_player());
    assert_eq!(living.name, "MassPlayMedia");
    assert_eq!(slot(&living, "entity_id"), Some("media_player.wohnzimmer_2"));

    let kitchen = one_with_area("Spiel Queen", "de", "kuche", home_with_kitchen_player());
    assert_eq!(kitchen.name, "MassPlayMedia");
    assert_eq!(slot(&kitchen, "entity_id"), Some("media_player.kuche_2"));
}

#[test]
fn explicit_music_area_overrides_assist_area() {
    let intent = one_with_area("Spiel Queen im Wohnzimmer", "de", "kuche", home_with_kitchen_player());
    assert_eq!(intent.name, "MassPlayMedia");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn play_on_plain_tagged_speaker_keeps_ha_search() {
    let mut home = default_home();
    home.entities.retain(|entity| entity.domain != "media_player");
    home.entities.push(entity("media_player.box", "Box", "media_player", None, Some("wohnzimmer"), &["box"], &["Musik"]));
    let parsed = intents("Spiel Queen", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(parsed[0].name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&parsed[0], "search_query"), Some("queen"));
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.box"));
}

#[test]
fn play_music_in_area_without_ma_unpauses_tagged_speaker() {
    let mut home = default_home();
    home.entities.retain(|entity| entity.domain != "media_player");
    home.entities.push(entity("media_player.e01", "Wohnzimmer Box", "media_player", None, Some("wohnzimmer"), &[], &["Musik"]));
    let parsed = intents("Spiele Musik im Wohnzimmer", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(parsed[0].name, "HassMediaUnpause");
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.e01"));
}

#[test]
fn media_commands_without_exposed_music_player_are_suppressed() {
    let mut home = home();
    home.entities.retain(|entity| entity.entity_id != "media_player.wohnzimmer_2");
    for text in [
        "Was läuft im Wohnzimmer",
        "Was kommt als nächstes Lied im Wohnzimmer",
        "Wie laut ist die Musik im Wohnzimmer",
        "Pausiere die Musik im Wohnzimmer",
        "Favorisiere den Titel im Wohnzimmer",
        "Spiel Queen im Wohnzimmer",
    ] {
        let parsed = intents(text, "de", &home, &mut Session::new());
        assert!(parsed.is_empty(), "{text}: {parsed:?}");
    }
}

#[test]
fn ambiguous_area_player_is_suppressed_without_tie_break() {
    let mut home = home();
    add_living_room_player(&mut home, "media_player.wohnzimmer_box", &["Musik"]);
    let parsed = intents("Spiel Queen im Wohnzimmer", "de", &home, &mut Session::new());
    assert!(parsed.is_empty(), "{parsed:?}");
}

#[test]
fn multiple_media_areas_are_suppressed() {
    let home = home_with_kitchen_player();
    let parsed = intents("Spiel Queen im Wohnzimmer und Küche", "de", &home, &mut Session::new());
    assert!(parsed.is_empty(), "{parsed:?}");
}

#[test]
fn unique_preferred_tag_breaks_area_tie() {
    let mut home = home();
    add_living_room_player(&mut home, "media_player.wohnzimmer_box", &["Musik", "preferred"]);
    let parsed = intents("Spiel Queen im Wohnzimmer", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.wohnzimmer_box"));
}

#[test]
fn session_last_player_breaks_area_tie_after_preferred_check() {
    let mut home = home();
    add_living_room_player(&mut home, "media_player.wohnzimmer_box", &["Musik"]);
    let mut session = Session::new();
    session.remember_entity("media_player.wohnzimmer_box");
    let parsed = intents("Pause im Wohnzimmer", "de", &home, &mut session);
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(parsed[0].name, "HassMediaPause");
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.wohnzimmer_box"));
}

#[test]
fn unexposed_preferred_player_never_wins() {
    let mut home = home();
    add_living_room_player(&mut home, "media_player.hidden_preferred", &["Musik", "preferred"]);
    home.assist = Some(["media_player.wohnzimmer_2".to_string()].into_iter().collect());
    let parsed = intents("Spiel Queen im Wohnzimmer", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn explicit_player_beats_preferred_area() {
    let intent = one_with_area("Spiel Queen auf der Soundbar", "de", "kuche", home_with_kitchen_player());
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
    assert_eq!(slot(&intent, "search_query"), Some("queen"));
}

#[test]
fn resolved_player_alias_is_removed_from_search_tail() {
    let mut home = home_with_kitchen_player();
    let kitchen = home.entities.iter_mut().find(|entity| entity.entity_id == "media_player.kuche_2").unwrap();
    kitchen.aliases.push("Küchenbereich".into());
    let parsed = intents("Spiel Queen auf Küchenbereich", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 1, "{parsed:?}");
    assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.kuche_2"));
    assert_eq!(slot(&parsed[0], "search_query"), Some("queen"));
}

#[test]
fn german_transport_imperatives_are_recognized() {
    for (text, expected) in [
        ("Pausiere die Musik", "HassMediaPause"),
        ("Pause die Musik", "HassMediaPause"),
        ("Musik stoppen", "HassMediaPause"),
        ("Wiedergabe stopp", "HassMediaPause"),
        ("Wiedergabe stoppen", "HassMediaPause"),
        ("Stopp die Wiedergabe", "HassMediaPause"),
        ("Nächster Titel", "HassMediaNext"),
        ("Nächste Musik", "HassMediaNext"),
    ] {
        let intent = one(text, "de");
        assert_eq!(intent.name, expected, "{text}: {intent:?}");
        assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
    }
}

#[test]
fn kitchen_status_phrases_emit_exactly_one_targeted_intent() {
    let mut home = home_with_kitchen_player();
    let kitchen = home.areas.iter_mut().find(|area| area.area_id == "kuche").unwrap();
    kitchen.aliases.push("Küchenbereich".into());
    for (text, expected) in
        [("Was kommt als nächstes Lied im Küchenbereich", "MassGetQueue"), ("Wie laut ist die Musik im Küchenbereich", "HassGetState")]
    {
        let parsed = intents(text, "de", &home, &mut Session::new());
        assert_eq!(parsed.len(), 1, "{text}: {parsed:?}");
        assert_eq!(parsed[0].name, expected);
        assert_eq!(slot(&parsed[0], "entity_id"), Some("media_player.kuche_2"));
        assert_eq!(slot(&parsed[0], "area"), None);
    }
}

#[test]
fn media_transport_does_not_fall_through_to_script() {
    let mut home = home();
    home.entities.retain(|entity| entity.domain != "media_player");
    home.entities.push(entity("script.pause", "Pause Musik", "script", None, None, &["pausiere musik"], &[]));
    let parsed = intents("Pausiere Musik", "de", &home, &mut Session::new());
    assert!(parsed.is_empty() || parsed.iter().all(|intent| intent.name == "KlarNoMusicPlayer"), "{parsed:?}");
}

#[test]
fn mass_intents_never_target_plain_media_players() {
    let home = home();
    let parsed = intents("Queue Bohemian Rhapsody on the TV", "en", &home, &mut Session::new());
    assert!(parsed.is_empty(), "{parsed:?}");
}

#[test]
fn transfer_requires_a_distinct_explicit_destination() {
    let home = home();
    let mut session = Session::new();
    session.remember_entity("media_player.wohnzimmer_2");

    let missing = intents("Verschiebe die Musik", "de", &home, &mut session);
    assert!(missing.is_empty(), "{missing:?}");

    let source_only = intents("Move music from the living room", "en", &home, &mut session);
    assert!(source_only.is_empty(), "{source_only:?}");

    let source_with_followup = intents("Move music from the living room and turn on the light in the kitchen", "en", &home, &mut session);
    assert!(source_with_followup.iter().all(|intent| intent.name != "MassTransferQueue"), "{source_with_followup:?}");
    assert!(source_with_followup.iter().any(|intent| intent.name == "HassTurnOn"), "{source_with_followup:?}");

    let same = intents("Verschiebe die Musik ins Wohnzimmer", "de", &home, &mut session);
    assert!(same.is_empty(), "{same:?}");
}

#[test]
fn protected_media_question_keeps_followup_clause() {
    let home = home();
    let parsed = intents("Wie laut ist die Musik und schalte das Licht im Wohnzimmer aus", "de", &home, &mut Session::new());
    assert_eq!(parsed.len(), 2, "{parsed:?}");
    assert!(parsed.iter().any(|intent| intent.name == "HassGetState"));
    assert!(parsed.iter().any(|intent| intent.name == "HassTurnOff"));
}

#[test]
fn protected_media_followup_skips_conjunction_inside_target_name() {
    let mut home = home();
    home.entities.push(entity(
        "light.wohn_und_esszimmer",
        "Wohn und Esszimmer Licht",
        "light",
        None,
        Some("wohnung"),
        &["wohn und esszimmer"],
        &[],
    ));
    let _lang = klar_nlu::lang::bind(&["de".into(), "en".into()]);
    let tokens: Vec<String> =
        "wie laut ist die musik im wohn und esszimmer und schalte das licht aus".split_whitespace().map(str::to_string).collect();
    let clauses = split_clauses(&tokens, &home);
    assert_eq!(clauses.len(), 2, "{clauses:?}");
    assert_eq!(clauses[0].join(" "), "wie laut ist die musik im wohn und esszimmer");
    assert_eq!(clauses[1].first().map(String::as_str), Some("schalte"));
}

#[test]
fn protected_media_followup_recursively_splits_remaining_actions() {
    let home = home();
    let _lang = klar_nlu::lang::bind(&["de".into(), "en".into()]);
    let tokens: Vec<String> =
        "wie laut ist die musik und schalte das licht aus und oeffne das rollo".split_whitespace().map(str::to_string).collect();
    let clauses = split_clauses(&tokens, &home);
    assert_eq!(clauses.len(), 3, "{clauses:?}");
    assert_eq!(clauses[0].join(" "), "wie laut ist die musik");
    assert_eq!(clauses[1].first().map(String::as_str), Some("schalte"));
    assert_eq!(clauses[2].first().map(String::as_str), Some("oeffne"));
}

#[test]
fn en_resume_music_unpauses_tagged_player() {
    let intent = one("resume music", "en");
    assert_eq!(intent.name, "HassMediaUnpause");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn en_play_artist_uses_mass() {
    let intent = one("play depeche mode", "en");
    assert_eq!(intent.name, "MassPlayMedia");
    assert!(slot(&intent, "media_id").unwrap_or("").contains("depeche"), "{intent:?}");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}

#[test]
fn de_play_artist_smoke() {
    let intent = one("spiel depeche mode", "de");
    assert_eq!(intent.name, "MassPlayMedia");
    assert!(slot(&intent, "media_id").unwrap_or("").contains("depeche"), "{intent:?}");
}

#[test]
fn protected_media_followup_splits_explicit_query() {
    let home = home();
    let _lang = klar_nlu::lang::bind(&["de".into(), "en".into()]);
    let tokens: Vec<String> =
        "wie laut ist die musik und was ist die temperatur im wohnzimmer".split_whitespace().map(str::to_string).collect();
    let clauses = split_clauses(&tokens, &home);
    assert_eq!(clauses.len(), 2, "{clauses:?}");
    assert_eq!(clauses[0].join(" "), "wie laut ist die musik");
    assert_eq!(clauses[1].first().map(String::as_str), Some("was"));
}
