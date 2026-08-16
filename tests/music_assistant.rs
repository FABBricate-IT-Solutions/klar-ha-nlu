use klar_nlu::home::default_home;
use klar_nlu::parse::parse;
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

fn slot<'a>(intent: &'a Intent, name: &str) -> Option<&'a str> {
    intent.slot(name)
}

#[test]
fn de_simple_play_uses_search_and_play_on_ma_player() {
    let intent = one("Spiel Queen", "de");
    assert_eq!(intent.name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&intent, "search_query"), Some("queen"));
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
    assert_eq!(intent.name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&intent, "search_query"), Some("chill"));
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

    let transfer = one("Move music to the living room", "en");
    assert_eq!(transfer.name, "MassTransferQueue");
    assert_eq!(slot(&transfer, "entity_id"), Some("media_player.wohnzimmer_2"));

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
    assert_eq!(living.name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&living, "entity_id"), Some("media_player.wohnzimmer_2"));

    let kitchen = one_with_area("Spiel Queen", "de", "kuche", home_with_kitchen_player());
    assert_eq!(kitchen.name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&kitchen, "entity_id"), Some("media_player.kuche_2"));
}

#[test]
fn explicit_music_area_overrides_assist_area() {
    let intent = one_with_area("Spiel Queen im Wohnzimmer", "de", "kuche", home_with_kitchen_player());
    assert_eq!(intent.name, "HassMediaSearchAndPlay");
    assert_eq!(slot(&intent, "entity_id"), Some("media_player.wohnzimmer_2"));
}
