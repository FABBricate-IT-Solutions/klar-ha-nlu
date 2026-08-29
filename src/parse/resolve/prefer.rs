use crate::home::expose::assist_visible;
use crate::home::policy::is_infra;
use crate::lang::catalog;
use crate::parse::normalize::{compact, fold_umlaut};
use crate::types::{EntityRec, HomeGraph};

pub(super) fn prefer_tv(tokens: &[String], home: &HomeGraph, candidates: &mut Vec<(f64, EntityRec)>) {
    if !tv_utterance(tokens) {
        return;
    }
    let mentioned: Vec<String> =
        home.areas.iter().filter(|area| area_mentioned(tokens, &area.area_id, home)).map(|area| area.area_id.clone()).collect();
    if mentioned.is_empty() {
        let aliased: Vec<(f64, EntityRec)> =
            candidates.iter().filter(|(_, entity)| looks_like_tv(entity) && has_tv_alias(entity)).cloned().collect();
        if aliased.len() == 1 {
            *candidates = aliased;
            return;
        }
        for entity in home.entities.iter().filter(|entity| looks_like_tv(entity) && assist_visible(entity, home) && !is_infra(entity)) {
            if !candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
                candidates.push((0.95, entity.clone()));
            }
        }
        let tvs: Vec<(f64, EntityRec)> = candidates.iter().filter(|(_, entity)| looks_like_tv(entity)).cloned().collect();
        if tvs.is_empty() {
            return;
        }
        let aliased: Vec<(f64, EntityRec)> = tvs.iter().filter(|(_, entity)| has_tv_alias(entity)).cloned().collect();
        if aliased.len() == 1 {
            *candidates = aliased;
            return;
        }
        let media: Vec<(f64, EntityRec)> = tvs.iter().filter(|(_, entity)| entity.domain == "media_player").cloned().collect();
        if media.len() == 1 {
            *candidates = media;
        }
        return;
    }
    for entity in home.entities.iter().filter(|entity| {
        looks_like_tv(entity)
            && assist_visible(entity, home)
            && !is_infra(entity)
            && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area))
    }) {
        if !candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
            candidates.push((0.95, entity.clone()));
        }
    }
    let in_area: Vec<(f64, EntityRec)> = candidates
        .iter()
        .filter(|(_, entity)| looks_like_tv(entity) && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area)))
        .cloned()
        .collect();
    if !in_area.is_empty() {
        let media: Vec<(f64, EntityRec)> = in_area.iter().filter(|(_, entity)| entity.domain == "media_player").cloned().collect();
        *candidates = if media.is_empty() { in_area } else { media };
        return;
    }
    let media: Vec<(f64, EntityRec)> = home
        .entities
        .iter()
        .filter(|entity| {
            entity.domain == "media_player"
                && assist_visible(entity, home)
                && !is_infra(entity)
                && entity.area.as_deref().is_some_and(|area| mentioned.iter().any(|id| id == area))
        })
        .map(|entity| (0.92, entity.clone()))
        .collect();
    *candidates = media;
}

fn area_mentioned(tokens: &[String], area_id: &str, home: &HomeGraph) -> bool {
    home.areas.iter().filter(|area| area.area_id == area_id).any(|area| {
        tokens.iter().any(|token| {
            token == area.area_id.as_str()
                || fold_umlaut(&area.name) == *token
                || area.aliases.iter().any(|alias| fold_umlaut(alias) == *token || compact(alias) == *token)
        })
    })
}

fn looks_like_tv(entity: &EntityRec) -> bool {
    if entity.domain != "media_player" && entity.domain != "switch" {
        return false;
    }
    let hay = format!("{} {} {}", entity.entity_id, entity.name, entity.aliases.join(" "));
    let folded = compact(&fold_umlaut(&hay));
    folded.contains("tv") || folded.contains("fernseher") || folded.contains("television")
}

fn tv_token(token: &str) -> bool {
    let folded = compact(&fold_umlaut(token));
    folded == "tv" || folded == "fernseher" || folded == "television" || catalog().tv_words().iter().any(|word| folded == compact(word))
}

fn tv_utterance(tokens: &[String]) -> bool {
    tokens.iter().any(|token| tv_token(token))
}

fn has_tv_alias(entity: &EntityRec) -> bool {
    entity.aliases.iter().any(|alias| tv_token(alias))
}

pub(super) fn prefer_entry_lock(tokens: &[String], home: &HomeGraph, candidates: &mut Vec<(f64, EntityRec)>) {
    let cat = catalog();
    if crate::parse::action::is_garage_cover(tokens) && !cat.any(tokens, cat.lock_verbs()) {
        return;
    }
    if cat.any(tokens, cat.sensor_words()) {
        return;
    }
    let lockish = cat.any(tokens, cat.lock_verbs()) || cat.any(tokens, cat.lock_nouns()) || cat.any(tokens, cat.door_nouns());
    if !lockish {
        return;
    }
    let has_lock = candidates.iter().any(|(_, entity)| entity.domain == "lock");
    if should_seed_locks(tokens, has_lock) {
        for entity in home.entities.iter().filter(|entity| entity.domain == "lock" && assist_visible(entity, home) && !is_infra(entity)) {
            if candidates.iter().any(|(_, existing)| existing.entity_id == entity.entity_id) {
                continue;
            }
            candidates.push((0.88, entity.clone()));
        }
    }
    let locks: Vec<(f64, EntityRec)> = candidates.iter().filter(|(_, entity)| entity.domain == "lock").cloned().collect();
    if locks.len() == 1 {
        *candidates = locks;
        return;
    }
    if locks.is_empty() {
        return;
    }
    let mentioned: Vec<(f64, EntityRec)> = locks.iter().filter(|(_, entity)| lock_mentioned(tokens, entity)).cloned().collect();
    if mentioned.len() >= 2 {
        *candidates = mentioned;
        return;
    }
    if mentioned.len() == 1 {
        *candidates = mentioned;
        return;
    }
    if session_lock_follow(tokens) {
        candidates.retain(|(_, entity)| entity.domain != "lock");
        return;
    }
    let entry: Vec<(f64, EntityRec)> = locks.iter().filter(|(_, entity)| is_entry_lock(entity)).cloned().collect();
    if entry.len() == 1 {
        *candidates = entry;
    }
}

pub(super) fn lock_mentioned(tokens: &[String], entity: &EntityRec) -> bool {
    let cat = catalog();
    tokens.iter().any(|token| {
        if token.len() <= 2 || cat.lock_nouns().contains(token.as_str()) {
            return false;
        }
        if cat.door_nouns().contains(token.as_str()) {
            return token.len() > 6 && exact_lock_label(entity, token);
        }
        if cat.entry_words().contains(token.as_str()) {
            return is_entry_lock(entity) && !garage_entry_phrase(tokens);
        }
        if (cat.garage_words().contains(token.as_str()) || token == "garage") && entity.area.as_deref() == Some("garage") {
            return true;
        }
        entity.area.as_deref().is_some_and(|area| area == token || fold_umlaut(area) == *token)
            || entity
                .entity_id
                .split_once('.')
                .map(|(_, rest)| rest.split(|c: char| !c.is_alphanumeric()).any(|part| part == token))
                .unwrap_or(false)
            || exact_lock_label(entity, token)
    })
}

fn exact_lock_label(entity: &EntityRec, token: &str) -> bool {
    let name = fold_umlaut(&entity.name);
    if name == *token {
        return true;
    }
    if !name.contains(|c: char| c.is_whitespace() || c == '-') && name.split(|c: char| !c.is_alphanumeric()).any(|part| part == token) {
        return true;
    }
    entity.aliases.iter().any(|alias| fold_umlaut(alias) == *token) || entity.tags.iter().any(|tag| fold_umlaut(tag) == *token)
}

pub(super) fn mentioned_locks(tokens: &[String], candidates: &[(f64, EntityRec)]) -> Option<Vec<EntityRec>> {
    let locks: Vec<EntityRec> = candidates
        .iter()
        .filter(|(_, entity)| entity.domain == "lock" && lock_mentioned(tokens, entity))
        .map(|(_, entity)| entity.clone())
        .collect();
    (locks.len() >= 2).then_some(locks)
}

pub(super) fn two_lock_rooms(tokens: &[String]) -> bool {
    let cat = catalog();
    let has_entry = cat.any(tokens, cat.entry_words());
    let has_garage = tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()));
    has_entry && has_garage && !garage_entry_phrase(tokens)
}

fn garage_entry_phrase(tokens: &[String]) -> bool {
    let cat = catalog();
    tokens.windows(2).any(|window| {
        (window[0] == "garage" || cat.garage_words().contains(window[0].as_str()))
            && (window[1] == "entry" || cat.entry_words().contains(window[1].as_str()))
    })
}

fn should_seed_locks(tokens: &[String], has_lock: bool) -> bool {
    if two_lock_rooms(tokens) {
        return true;
    }
    let cat = catalog();
    if cat.any(tokens, cat.lock_verbs()) && cat.any(tokens, cat.conjunctions()) {
        return true;
    }
    if has_lock {
        return false;
    }
    let door = cat.any(tokens, cat.door_nouns());
    let lock_noun = cat.any(tokens, cat.lock_nouns());
    if !door && !lock_noun {
        return false;
    }
    let grounded = door
        || cat.any(tokens, cat.entry_words())
        || tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()));
    if pronoun_follow(tokens) && !grounded {
        return false;
    }
    true
}

fn pronoun_follow(tokens: &[String]) -> bool {
    tokens.iter().any(|token| matches!(token.as_str(), "it" | "that" | "this" | "es" | "ihn" | "sie"))
}

fn session_lock_follow(tokens: &[String]) -> bool {
    let cat = catalog();
    !cat.any(tokens, cat.door_nouns())
        && !cat.any(tokens, cat.entry_words())
        && !tokens.iter().any(|token| token == "garage" || cat.garage_words().contains(token.as_str()))
}

fn is_entry_lock(entity: &EntityRec) -> bool {
    let cat = catalog();
    if entity.area.as_deref() == Some("garage") || entity.entity_id.contains("garage") {
        return false;
    }
    entity.entity_id.contains("front")
        || entity.area.as_deref().is_some_and(|area| area == "entryway" || area == "entry")
        || cat
            .entry_words()
            .iter()
            .any(|word| fold_umlaut(&entity.name).contains(word) || entity.aliases.iter().any(|alias| fold_umlaut(alias).contains(word)))
}

#[cfg(test)]
mod tests {
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
        let scene = parse("Aktiviere Alles aus", &home, &mut Session::new(), &[], &settings);
        assert_eq!(scene.intents.first().and_then(|intent| intent.slot("entity_id")), Some("scene.alles_aus"), "{scene:?}");
        let en = Settings { languages: vec!["de".into(), "en".into()], ..Settings::default() };
        let english = parse("Activate All off", &home, &mut Session::new(), &[], &en);
        assert_eq!(english.intents.first().and_then(|intent| intent.slot("entity_id")), Some("scene.alles_aus"), "{english:?}");
        let tv = parse("Mach den Fernseher im Wohnzimmer an.", &home, &mut Session::new(), &[], &settings);
        assert_eq!(tv.intents.first().and_then(|intent| intent.slot("entity_id")), Some("media_player.wohnzimmer_tv"), "{tv:?}");
        assert_ne!(tv.intents.first().and_then(|intent| intent.slot("entity_id")), Some("switch.schlafzimmer_tv"), "{tv:?}");
        let bare = parse("Mach den Fernseher an.", &home, &mut Session::new(), &[], &settings);
        assert!(!bare.clarify, "{bare:?}");
        assert_eq!(bare.intents.first().and_then(|intent| intent.slot("entity_id")), Some("switch.schlafzimmer_tv"), "{bare:?}");
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
}
