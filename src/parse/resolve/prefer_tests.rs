use crate::parse::parse;
use crate::session::Session;
use crate::types::{AreaRec, EntityRec, HomeGraph, Settings};

fn lock_home() -> HomeGraph {
    HomeGraph {
        areas: vec![
            AreaRec { area_id: "entryway".into(), name: "Eingang".into(), aliases: vec!["eingang".into()], floor_id: None },
            AreaRec { area_id: "garage".into(), name: "Garage".into(), aliases: vec!["garage".into()], floor_id: None },
        ],
        entities: vec![
            EntityRec {
                entity_id: "lock.front_door".into(),
                name: "Haustür".into(),
                domain: "lock".into(),
                platform: None,
                area: Some("entryway".into()),
                aliases: vec!["haustuer".into()],
                tags: Vec::new(),
            },
            EntityRec {
                entity_id: "lock.garage_entry".into(),
                name: "Garagentür".into(),
                domain: "lock".into(),
                platform: None,
                area: Some("garage".into()),
                aliases: vec![],
                tags: Vec::new(),
            },
            EntityRec {
                entity_id: "light.garage_light".into(),
                name: "Garage Licht".into(),
                domain: "light".into(),
                platform: None,
                area: Some("garage".into()),
                aliases: vec![],
                tags: Vec::new(),
            },
            EntityRec {
                entity_id: "cover.garage_door".into(),
                name: "Garagenrollo".into(),
                domain: "cover".into(),
                platform: None,
                area: Some("garage".into()),
                aliases: vec!["garagenrollo".into()],
                tags: Vec::new(),
            },
        ],
        ..HomeGraph::default()
    }
}

#[test]
fn two_named_locks_stay_both() {
    let result = parse("schliess tuer eingang und garage", &lock_home(), &mut Session::new(), &[], &Settings::pinned("de"));
    let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
    assert!(ids.contains(&"lock.front_door"), "{result:?}");
    assert!(ids.contains(&"lock.garage_entry"), "{result:?}");
    assert!(!ids.contains(&"light.garage_light"), "{result:?}");
    assert!(!ids.contains(&"cover.garage_door"), "{result:?}");
}

#[test]
fn generated_de_home_locks_both() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/full_home/de/home_config.yaml");
    let home = crate::home::load_home_config(&path).expect("de home");
    for sentence in ["schliess tuer eingang und garage", "schliess haustuer und garagentor"] {
        let result = parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned("de"));
        let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
        assert!(!result.clarify, "{sentence}: {result:?}");
        assert!(ids.contains(&"lock.front_door"), "{sentence}: {result:?}");
        assert!(ids.contains(&"lock.garage_entry"), "{sentence}: {result:?}");
        assert!(!ids.contains(&"cover.garage_door"), "{sentence}: {result:?}");
    }
}

#[test]
fn lights_off_and_front_lock_keep_both() {
    let mut home = lock_home();
    home.areas.push(AreaRec {
        area_id: "living".into(),
        name: "Wohnzimmer".into(),
        aliases: vec!["wohnzimmer".into(), "salon".into()],
        floor_id: None,
    });
    home.entities.push(EntityRec {
        entity_id: "light.living_ceiling".into(),
        name: "Wohnzimmer Decke".into(),
        domain: "light".into(),
        platform: None,
        area: Some("living".into()),
        aliases: vec![],
        tags: Vec::new(),
    });
    let result = parse("licht wohnzimmer aus und schliess die haustuer", &home, &mut Session::new(), &[], &Settings::pinned("de"));
    assert!(!result.clarify, "{result:?}");
    assert!(result.intents.iter().any(|intent| intent.name == "HassTurnOff"), "{result:?}");
    assert!(result.intents.iter().any(|intent| intent.slot("entity_id") == Some("lock.front_door")), "{result:?}");
}

#[test]
fn query_track_room_is_now_playing() {
    let mut home = lock_home();
    home.areas.push(AreaRec {
        area_id: "living".into(),
        name: "Salon".into(),
        aliases: vec!["salon".into(), "ribingu".into()],
        floor_id: None,
    });
    home.entities.push(EntityRec {
        entity_id: "media_player.living_music".into(),
        name: "salon musique".into(),
        domain: "media_player".into(),
        platform: Some("music_assistant".into()),
        area: Some("living".into()),
        aliases: vec!["musique".into()],
        tags: vec!["musique".into()],
    });
    for (lang, sentence) in [("fr", "quel track salon"), ("ja", "nani track ribingu")] {
        let result = parse(sentence, &home, &mut Session::new(), &[], &Settings::pinned(lang));
        assert!(!result.clarify, "{lang} {sentence}: {result:?}");
        assert_eq!(result.intents.len(), 1, "{lang} {sentence}: {result:?}");
        assert_eq!(result.intents[0].name, "HassGetState", "{lang} {sentence}: {result:?}");
        assert_eq!(result.intents[0].slot("entity_id"), Some("media_player.living_music"), "{lang} {sentence}: {result:?}");
    }
}

#[test]
fn generated_off_and_lock_door_keeps_both() {
    let mut home = lock_home();
    home.areas.push(AreaRec { area_id: "living".into(), name: "Salon".into(), aliases: vec!["salon".into()], floor_id: None });
    home.entities.push(EntityRec {
        entity_id: "light.living_ceiling".into(),
        name: "Salon".into(),
        domain: "light".into(),
        platform: None,
        area: Some("living".into()),
        aliases: vec![],
        tags: Vec::new(),
    });
    let result = parse("eteins lumiere salon et verrouille porte", &home, &mut Session::new(), &[], &Settings::pinned("fr"));
    assert!(!result.clarify, "{result:?}");
    assert!(result.intents.iter().any(|intent| intent.name == "HassTurnOff"), "{result:?}");
    assert!(result.intents.iter().any(|intent| intent.slot("entity_id") == Some("lock.front_door")), "{result:?}");
}

#[test]
fn generated_front_and_garage_locks_stay_both() {
    let result = parse("verrouille serrure entree et garage", &lock_home(), &mut Session::new(), &[], &Settings::pinned("fr"));
    let ids: Vec<_> = result.intents.iter().filter_map(|intent| intent.slot("entity_id")).collect();
    assert!(ids.contains(&"lock.front_door"), "{result:?}");
    assert!(ids.contains(&"lock.garage_entry"), "{result:?}");
}

#[test]
fn garage_lock_query_stays_garage() {
    let result = parse("Status Schloss Garage", &lock_home(), &mut Session::new(), &[], &Settings::pinned("de"));
    assert!(!result.clarify, "{result:?}");
    assert_eq!(result.intents.len(), 1, "{result:?}");
    assert_eq!(result.intents[0].slot("entity_id"), Some("lock.garage_entry"), "{result:?}");
}

#[test]
fn wohnung_kitchen_music_and_all_off_scene() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/datasets/wohnung_mittel/home_config.yaml");
    let home = crate::home::load_home_config(&path).expect("wohnung home");
    let settings = Settings { languages: vec!["de".into()], ..Settings::default() };
    let music = parse("Spiel Musik in der Küche.", &home, &mut Session::new(), &[], &settings);
    assert!(!music.clarify, "{music:?}");
    assert_eq!(music.intents.first().and_then(|intent| intent.slot("entity_id")), Some("media_player.kuchenbereich"), "{music:?}");
    let bare_music = parse("Musik in der Küche", &home, &mut Session::new(), &[], &settings);
    assert!(!bare_music.clarify, "{bare_music:?}");
    assert_eq!(bare_music.intents.first().map(|intent| intent.name.as_str()), Some("HassMediaUnpause"), "{bare_music:?}");
    assert_eq!(
        bare_music.intents.first().and_then(|intent| intent.slot("entity_id")),
        Some("media_player.kuchenbereich"),
        "{bare_music:?}"
    );
    let scene = parse("Aktiviere Alles aus", &home, &mut Session::new(), &[], &settings);
    assert_eq!(scene.intents.first().and_then(|intent| intent.slot("entity_id")), Some("scene.alles_aus"), "{scene:?}");
    let en = Settings { languages: vec!["de".into(), "en".into()], ..Settings::default() };
    let english = parse("Activate All off", &home, &mut Session::new(), &[], &en);
    assert_eq!(english.intents.first().and_then(|intent| intent.slot("entity_id")), Some("scene.alles_aus"), "{english:?}");
    let tv = parse("Mach den Fernseher im Wohnzimmer an.", &home, &mut Session::new(), &[], &settings);
    assert_eq!(tv.intents.first().and_then(|intent| intent.slot("entity_id")), Some("media_player.wohnzimmer_tv"), "{tv:?}");
    assert_ne!(tv.intents.first().and_then(|intent| intent.slot("entity_id")), Some("switch.schlafzimmer_tv"), "{tv:?}");
    let bare_tv = parse("Fernseher im Wohnzimmer", &home, &mut Session::new(), &[], &settings);
    assert!(!bare_tv.clarify, "{bare_tv:?}");
    assert_eq!(bare_tv.intents.first().map(|intent| intent.name.as_str()), Some("HassTurnOn"), "{bare_tv:?}");
    assert_eq!(bare_tv.intents.first().and_then(|intent| intent.slot("entity_id")), Some("media_player.wohnzimmer_tv"), "{bare_tv:?}");
    assert!(!bare_tv.speech.to_lowercase().contains("licht"), "{bare_tv:?}");
    let bare = parse("Mach den Fernseher an.", &home, &mut Session::new(), &[], &settings);
    assert!(!bare.clarify, "{bare:?}");
    assert_eq!(bare.intents.first().and_then(|intent| intent.slot("entity_id")), Some("switch.schlafzimmer_tv"), "{bare:?}");
    let heat = parse("Heizung Wohnzimmer auf 21", &home, &mut Session::new(), &[], &settings);
    assert!(!heat.clarify, "{heat:?}");
    assert_eq!(heat.intents.first().map(|intent| intent.name.as_str()), Some("HassClimateSetTemperature"), "{heat:?}");
    assert_eq!(heat.intents.first().and_then(|intent| intent.slot("temperature")), Some("21"), "{heat:?}");
    assert_eq!(heat.intents.first().and_then(|intent| intent.slot("entity_id")), Some("climate.better_thermostat_wohnzimmer"), "{heat:?}");
    let status = parse("Status TV", &home, &mut Session::new(), &[], &settings);
    assert!(!status.clarify, "{status:?}");
    assert_eq!(status.intents.first().and_then(|intent| intent.slot("entity_id")), Some("switch.schlafzimmer_tv"), "{status:?}");
}

#[test]
fn named_room_tv_does_not_fall_back_to_bedroom() {
    let mut home = lock_home();
    home.areas.push(AreaRec {
        area_id: "wohnzimmer".into(),
        name: "Wohnzimmer".into(),
        aliases: vec!["wohnzimmer".into(), "living".into()],
        floor_id: None,
    });
    home.areas.push(AreaRec {
        area_id: "schlafzimmer".into(),
        name: "Schlafzimmer".into(),
        aliases: vec!["schlafzimmer".into(), "bedroom".into()],
        floor_id: None,
    });
    home.entities.push(EntityRec {
        entity_id: "switch.schlafzimmer_tv".into(),
        name: "Schlafzimmer TV".into(),
        domain: "switch".into(),
        platform: None,
        area: Some("schlafzimmer".into()),
        aliases: vec!["Fernseher".into()],
        tags: Vec::new(),
    });
    home.entities.push(EntityRec {
        entity_id: "media_player.wohnzimmer".into(),
        name: "Wohnzimmer Player".into(),
        domain: "media_player".into(),
        platform: Some("music_assistant".into()),
        area: Some("wohnzimmer".into()),
        aliases: vec![],
        tags: Vec::new(),
    });
    home.entities.push(EntityRec {
        entity_id: "light.wohnzimmer".into(),
        name: "Wohnzimmer Licht".into(),
        domain: "light".into(),
        platform: None,
        area: Some("wohnzimmer".into()),
        aliases: vec![],
        tags: Vec::new(),
    });
    let result = parse("Mach den Fernseher im Wohnzimmer an.", &home, &mut Session::new(), &[], &Settings::pinned("de"));
    let id = result.intents.first().and_then(|intent| intent.slot("entity_id"));
    assert_ne!(id, Some("switch.schlafzimmer_tv"), "{result:?}");
    assert_ne!(id, Some("light.wohnzimmer"), "{result:?}");
    assert_eq!(id, Some("media_player.wohnzimmer"), "{result:?}");
}

#[test]
fn lock_it_follows_last_door() {
    let home = lock_home();
    let mut session = Session::new();
    let first = parse("What's the status of the Front Door?", &home, &mut session, &[], &Settings::pinned("en"));
    assert_eq!(first.intents[0].slot("entity_id"), Some("lock.front_door"), "{first:?}");
    let second = parse("Please lock it for me.", &home, &mut session, &[], &Settings::pinned("en"));
    assert!(!second.clarify, "{second:?}");
    assert_eq!(second.intents[0].slot("entity_id"), Some("lock.front_door"), "{second:?}");
}
