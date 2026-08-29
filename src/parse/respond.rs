use crate::lang::catalog;
use crate::parse::normalize::compact;
use crate::parse::normalize::umlaut_eq;
use crate::types::{HomeGraph, Intent, Personality};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

static WRAP_TICK: AtomicU64 = AtomicU64::new(0);

fn speech() -> &'static crate::lang::Speech {
    catalog().speech()
}

/// Speech for `/api/v2/parse` and Wyoming. Assist overwrites this with
/// `speech.py` after the Home Assistant intent so the spoken line matches
/// what actually ran. Keep Butler style and infra filtering aligned there.
pub fn speak(intents: &[Intent], personality: Personality, clarify: bool, home: Option<&HomeGraph>) -> String {
    if clarify {
        return speech().need_which.to_string();
    }
    if intents.is_empty() {
        return speak_unknown();
    }
    wrap(personality, &describe_all(intents, home))
}

pub fn speak_unknown() -> String {
    speech().unknown.to_string()
}

pub fn speak_need_target(off: bool) -> String {
    if off {
        speech().need_off.to_string()
    } else {
        speech().need_on.to_string()
    }
}

pub fn speak_correction() -> String {
    speech().correction.to_string()
}

pub fn speak_clarify(names: &[String], home: Option<&HomeGraph>) -> String {
    let pack = speech();
    let labels: Vec<String> = names.iter().map(|id| clarify_label(id, home)).collect();
    pack.clarify.replace("{names}", &labels.join(pack.clarify_or))
}

fn describe_all(intents: &[Intent], home: Option<&HomeGraph>) -> String {
    if intents.len() > 1 && intents.iter().all(|i| i.name == intents[0].name) {
        let names: Vec<String> = intents.iter().map(|i| spoken_where(i, home)).filter(|s| !s.is_empty()).collect();
        if names.len() > 1 {
            if let Some(group) = describe_group(&intents[0].name, &names) {
                return group;
            }
        }
    }
    intents.iter().map(|i| describe(i, home)).collect::<Vec<_>>().join(" ")
}

fn describe_group(name: &str, wheres: &[String]) -> Option<String> {
    let pack = speech();
    let joined = join_names(wheres);
    let template = match name {
        "HassTurnOn" => pack.group_on,
        "HassTurnOff" => pack.group_off,
        _ => return None,
    };
    Some(template.replace("{names}", &joined))
}

fn join_names(names: &[String]) -> String {
    let conj = speech().and_join;
    match names {
        [] => String::new(),
        [one] => one.clone(),
        [.., last] => format!("{}{conj}{last}", names[..names.len() - 1].join(", ")),
    }
}

fn describe(intent: &Intent, home: Option<&HomeGraph>) -> String {
    let pack = speech();
    let where_ = spoken_where(intent, home);
    let area = intent.slot("area").unwrap_or("");
    let target = or_home(&where_);
    let fill = |template: &str| template.replace("{target}", &target).replace("{loc}", &loc(area, home)).replace("{name}", &intent.name);
    match intent.name.as_str() {
        "HassTurnOn" if looks_started(intent.slot("entity_id").unwrap_or("")) => fill(pack.turn_on_scene),
        "HassTurnOn" => fill(pack.turn_on),
        "HassTurnOff" => fill(pack.turn_off),
        "HassToggle" => fill(pack.toggle),
        "HassLightSet" => match (intent.slot("color"), intent.slot("brightness"), intent.slot("brightness_step")) {
            (Some(color), None, _) => fill(pack.light_color).replace("{color}", &catalog().color_spoken(color)),
            (_, Some(brightness), _) => fill(pack.light_set).replace("{n}", brightness),
            (_, _, Some(_)) => fill(pack.get_state),
            _ => fill(pack.light_set).replace("{n}", "?"),
        },
        "HassClimateSetTemperature" => {
            let noun = climate_noun(intent);
            let subject = if climate_named(&target, noun) { target } else { format!("{noun} {target}") };
            pack.climate_set.replace("{noun} {target}", &subject).replace("{n}", intent.slot("temperature").unwrap_or("?"))
        }
        "HassClimateGetTemperature" => fill(pack.get_temp),
        "HassGetState" if intent.slot("device_class") == Some("temperature") => fill(pack.get_temp),
        "HassGetState" => fill(pack.get_state),
        "HassMediaPause" => pack.media_pause.to_string(),
        "HassMediaUnpause" => pack.media_play.to_string(),
        "HassMediaNext" => pack.media_next.to_string(),
        "HassMediaPrevious" => pack.media_previous.to_string(),
        "HassMediaPlayerMute" => pack.media_mute.to_string(),
        "HassMediaPlayerUnmute" => pack.media_unmute.to_string(),
        "HassSetVolume" | "HassSetVolumeRelative" => pack.media_volume.to_string(),
        "HassMediaSearchAndPlay" | "MassPlayMedia" => pack.media_search.to_string(),
        "MassTransferQueue" => pack.media_transfer.to_string(),
        "MassFavorite" => pack.media_favorite.to_string(),
        "HassFanSetSpeed" => pack.fan_set.replace("{n}", intent.slot("percentage").unwrap_or("?")),
        "HassVacuumStart" => pack.vacuum_start.replace("{target}", &vacuum_name(&where_)),
        "HassVacuumReturnToBase" => pack.vacuum_dock.replace("{target}", &vacuum_name(&where_)),
        "HassStartTimer" => pack.timer_start.to_string(),
        "HassCancelTimer" => pack.timer_cancel.to_string(),
        "HassPauseTimer" => pack.timer_pause.to_string(),
        "HassListAddItem" | "HassShoppingListAddItem" => pack.list_add.to_string(),
        "KlarGetCalendarEvents" => pack.calendar_list.replace("{items}", "").replace("{count}", "0"),
        "KlarCreateCalendarEvent" if intent.slot("need") == Some("title") => pack.calendar_need_title.to_string(),
        "KlarCreateCalendarEvent" if intent.slot("need") == Some("when") => pack.calendar_need_when.to_string(),
        "KlarCreateCalendarEvent" => pack
            .calendar_created
            .replace("{summary}", intent.slot("summary").unwrap_or(""))
            .replace("{when}", intent.slot("day").unwrap_or("")),
        "KlarDeleteCalendarEvent" if intent.slot("need") == Some("which") => pack.calendar_which.to_string(),
        "KlarDeleteCalendarEvent" => pack.calendar_deleted.replace("{summary}", intent.slot("summary").unwrap_or("")),
        "KlarMoveCalendarEvent" if intent.slot("need") == Some("when") => pack.calendar_need_when.to_string(),
        "KlarMoveCalendarEvent" if intent.slot("need") == Some("which") => pack.calendar_which.to_string(),
        "KlarMoveCalendarEvent" => pack
            .calendar_moved
            .replace("{summary}", intent.slot("summary").unwrap_or(""))
            .replace("{when}", intent.slot("day").unwrap_or("")),
        "KlarNoMusicPlayer" => pack.no_music_player.to_string(),
        other => pack.done.replace("{name}", other),
    }
}

fn spoken_where(intent: &Intent, home: Option<&HomeGraph>) -> String {
    if let Some(id) = intent.slot("entity_id") {
        if looks_started(id) {
            return scene_label(id, home);
        }
        if let Some(ent) = home.and_then(|h| h.entities.iter().find(|e| e.entity_id == id)) {
            let name = if looks_like_entity_id(&ent.name) { object_id(id) } else { ent.name.clone() };
            let label = device_label(&name, &ent.domain);
            if ent.domain == "light" {
                if let Some(area) = ent.area.as_deref() {
                    let room = pretty_room(area, home);
                    if light_needs_room_label(&label, &room) {
                        return area_label(area, "light", &intent.name, home);
                    }
                }
            }
            return label;
        }
        return device_label(&object_id(id), domain_of(id));
    }
    if let Some(area) = intent.slot("area") {
        return area_label(area, intent.slot("domain").unwrap_or(""), &intent.name, home);
    }
    if let Some(floor) = intent.slot("floor") {
        if let Some(record) = home.and_then(|graph| graph.floor(floor)) {
            return record.name.clone();
        }
        return title_word(&floor.replace('_', " "));
    }
    String::new()
}

fn light_needs_room_label(label: &str, room: &str) -> bool {
    let folded = compact(label);
    let room = compact(room);
    if room.is_empty() || folded.contains(&room) || umlaut_eq(&folded, &room) {
        return false;
    }
    catalog().light_nouns().iter().any(|noun| folded == compact(noun) || folded.contains(&compact(noun)))
        || catalog().light_singular().iter().any(|noun| folded == compact(noun) || folded.contains(&compact(noun)))
}

fn looks_started(id: &str) -> bool {
    id.starts_with("scene.") || id.starts_with("script.")
}

fn scene_label(id: &str, home: Option<&HomeGraph>) -> String {
    home.and_then(|h| h.entities.iter().find(|e| e.entity_id == id))
        .map(|e| pretty_device(&e.name))
        .unwrap_or_else(|| pretty_device(&object_id(id)))
}

fn device_label(name: &str, domain: &str) -> String {
    let pack = speech();
    let pretty = pretty_device(name);
    let folded = compact(&pretty);
    let named = catalog().light_nouns().iter().any(|n| folded.contains(n))
        || catalog().named_device().iter().any(|n| folded.contains(n))
        || catalog().light_singular().iter().any(|n| folded.contains(n));
    if domain == "light" && !named {
        format!("{pretty}{}", pack.light_suffix)
    } else {
        pretty
    }
}

fn area_label(area: &str, domain: &str, intent: &str, home: Option<&HomeGraph>) -> String {
    let light = domain == "light" || matches!(intent, "HassTurnOn" | "HassTurnOff" | "HassToggle" | "HassLightSet");
    if light && domain != "climate" && domain != "fan" && domain != "media_player" && domain != "switch" {
        speech().area_light.replace("{loc}", &loc(area, home))
    } else {
        pretty_room(area, home)
    }
}

fn clarify_label(id: &str, home: Option<&HomeGraph>) -> String {
    if let Some(ent) = home.and_then(|h| h.entities.iter().find(|e| e.entity_id == id)) {
        let pretty = pretty_device(&ent.name);
        if !pretty.is_empty() {
            return pretty;
        }
    }
    pretty_device(&object_id(id))
}

fn object_id(id: &str) -> String {
    id.rsplit('.').next().unwrap_or(id).replace('_', " ")
}

fn domain_of(id: &str) -> &str {
    id.split('.').next().unwrap_or("")
}

fn climate_noun(intent: &Intent) -> &'static str {
    let pack = speech();
    let id = intent.slot("entity_id").unwrap_or("");
    let cool = catalog().climate_cool().iter().any(|w| id.contains(w)) && !id.contains("thermostat");
    if cool {
        pack.cool_noun
    } else {
        pack.heat_noun
    }
}

fn climate_named(target: &str, noun: &str) -> bool {
    let noun_folded = compact(noun);
    crate::parse::normalize::tokenize(target).into_iter().any(|folded| {
        folded == noun_folded
            || catalog().climate_nouns().contains(folded.as_str())
            || catalog().climate_cool().contains(folded.as_str())
            || catalog().climate_heat().contains(folded.as_str())
            || matches!(folded.as_str(), "heizung" | "heat" | "heater" | "thermostat" | "klima" | "klimaanlage" | "ac" | "aircon")
    })
}

fn looks_like_entity_id(value: &str) -> bool {
    value.contains('.') && !value.contains(' ') && value.split('.').count() == 2
}

fn vacuum_name(where_: &str) -> String {
    let pack = speech();
    if where_.is_empty() || where_ == "Zuhause" || where_ == "home" {
        pack.vacuum_default.to_string()
    } else {
        where_.to_string()
    }
}

fn pretty_device(raw: &str) -> String {
    let parts: Vec<&str> = raw.split_whitespace().filter(|p| !p.is_empty()).collect();
    let tail = parts.last().copied().unwrap_or("");
    let light = catalog().light_nouns().contains(tail) || catalog().light_plural().contains(tail);
    let all = parts.first().is_some_and(|p| catalog().all_words().contains(p));
    if light && !all && parts.len() >= 2 {
        let head = parts[..parts.len() - 1].join("");
        return title_word(&format!("{head}{tail}"));
    }
    parts
        .iter()
        .enumerate()
        .map(
            |(i, part)| {
                if i > 0 && (catalog().is_conj(part) || catalog().is_filler(part)) {
                    (*part).to_string()
                } else {
                    title_word(part)
                }
            },
        )
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

fn loc(area: &str, home: Option<&HomeGraph>) -> String {
    let pack = speech();
    if area.is_empty() {
        return pack.loc_home.to_string();
    }
    let room = pretty_room(area, home);
    let folded = compact(area);
    let id_folded = area_record(area, home).map(|rec| compact(&rec.area_id)).unwrap_or_else(|| folded.clone());
    let feminine = pack.loc_der_rooms.contains(&folded.as_str())
        || pack.loc_der_rooms.contains(&id_folded.as_str())
        || room.ends_with('e')
        || room.ends_with('E');
    let template = if feminine { pack.loc_in_der } else { pack.loc_in };
    template.replace("{room}", &room)
}

fn area_record<'a>(area: &'a str, home: Option<&'a HomeGraph>) -> Option<&'a crate::types::AreaRec> {
    home.and_then(|graph| {
        graph.areas.iter().find(|rec| {
            rec.area_id == area || umlaut_eq(&compact(&rec.area_id), &compact(area)) || umlaut_eq(&compact(&rec.name), &compact(area))
        })
    })
}

fn pretty_room(area: &str, home: Option<&HomeGraph>) -> String {
    let pack = speech();
    let rec = area_record(area, home);
    let folded = compact(area);
    if let Some(name) = pack.room_name(&folded).or_else(|| rec.and_then(|item| pack.room_name(&compact(&item.area_id)))) {
        return name.to_string();
    }
    if let Some(rec) = rec {
        return rec.name.clone();
    }
    title_word(&area.replace('_', " "))
}

fn or_home(s: &str) -> String {
    if s.is_empty() {
        speech().or_home.to_string()
    } else {
        s.to_string()
    }
}

fn wrap(personality: Personality, body: &str) -> String {
    if matches!(personality, Personality::Default) {
        return body.to_string();
    }
    let key = match personality {
        Personality::Butler => "butler",
        Personality::Locker => "locker",
        Personality::Fuersorglich => "fuersorglich",
        Personality::Party => "party",
        Personality::Grantig => "grantig",
        Personality::Sarkastisch => "sarkastisch",
        Personality::Pirat => "pirat",
        Personality::Hippie => "hippie",
        Personality::Gollum => "gollum",
        Personality::Jarvis => "jarvis",
        Personality::Default => return body.to_string(),
    };
    let prefixes = speech().personality_prefixes(key);
    if prefixes.is_empty() {
        return body.to_string();
    }
    let prefix = prefixes[pick_variant(body, prefixes.len())];
    if prefix.is_empty() {
        return body.to_string();
    }
    format!("{prefix}{body}")
}

fn pick_variant(body: &str, n: usize) -> usize {
    if n <= 1 {
        return 0;
    }
    let tick = WRAP_TICK.fetch_add(1, Ordering::Relaxed);
    let mut hasher = DefaultHasher::new();
    body.hash(&mut hasher);
    tick.hash(&mut hasher);
    (hasher.finish() as usize) % n
}

#[cfg(test)]
#[path = "respond_tests.rs"]
mod tests;
