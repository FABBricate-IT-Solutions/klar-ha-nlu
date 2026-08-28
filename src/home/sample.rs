use crate::types::{AreaRec, EntityRec, HomeGraph};

pub fn default_home() -> HomeGraph {
    let areas = vec![
        area("wohnzimmer", "Wohnzimmer", &["wohnraum", "wohn", "living", "livingroom", "lounge"]),
        area("esszimmer", "Esszimmer", &["ess", "dining", "diningroom"]),
        area("schlafzimmer", "Schlafzimmer", &["schlaf", "bedroom", "master"]),
        area("kuche", "Küche", &["kueche", "kuche", "kitchen"]),
        area("badezimmer", "Badezimmer", &["bad", "bathroom", "bath"]),
        area("arbeitszimmer", "Arbeitszimmer", &["buero", "office", "study"]),
        area("flur", "Flur", &["diele", "hallway", "hall", "corridor"]),
        area("balkon", "Balkon", &["draussen", "aussen", "terrasse", "balcony", "terrace"]),
        area("wohnung", "Wohnung", &["haus", "zuhause", "hier", "ueberall", "home", "house", "apartment", "everywhere"]),
    ];
    let entities = vec![
        ent("light.wohnzimmer", "Wohnzimmer Licht", "light", "wohnzimmer", &["wohnzimmer"]),
        ent("light.esszimmer", "Esszimmer Licht", "light", "esszimmer", &["esszimmer"]),
        ent("light.arbeitszimmer", "Arbeitszimmer", "light", "arbeitszimmer", &["arbeitszimmer"]),
        ent("light.kuche_kuche", "Küche Licht", "light", "kuche", &["kueche", "kuche", "kitchen"]),
        ent("light.schlafzimmer_kugel", "Kugel", "light", "schlafzimmer", &["kugel"]),
        ent("light.schlafzimmer_decke", "Deckenlampe", "light", "schlafzimmer", &["deckenlampe", "decke"]),
        ent("light.schlafzimmer_licht", "Schlafzimmer Licht", "light", "schlafzimmer", &["schlafzimmer licht"]),
        ent("light.alle_lichter", "Alle Lichter", "light", "wohnung", &["alle", "ueberall", "all", "everywhere"]),
        ent("climate.better_thermostat_wohnzimmer", "Heizung Wohnzimmer", "climate", "wohnzimmer", &["heizung wohnzimmer"]),
        ent("climate.better_thermostat_esszimmer", "Heizung Esszimmer", "climate", "esszimmer", &["heizung esszimmer"]),
        ent("climate.better_thermostat_schlafzimmer", "Heizung Schlafzimmer", "climate", "schlafzimmer", &["heizung schlafzimmer"]),
        ent("climate.better_thermostat_badezimmer", "Heizung Bad", "climate", "badezimmer", &["heizung bad"]),
        ent("climate.schlafzimmer_ac", "Klimaanlage", "climate", "schlafzimmer", &["klima", "ac"]),
        ent("vacuum.r2d2", "R2D2", "vacuum", "wohnzimmer", &["staubsauger", "sauger", "saugroboter"]),
        ent("switch.pc_steckdose", "PC Steckdose", "switch", "arbeitszimmer", &["pc"]),
        ent("switch.schlafzimmer_tv", "Schlafzimmer TV", "switch", "schlafzimmer", &["tv"]),
        ent("switch.badezimmer_waschmaschine", "Waschmaschine", "switch", "badezimmer", &["waschmaschine"]),
        ent("switch.kuche_trockner", "Trockner", "switch", "kuche", &["trockner"]),
        ent("switch.kuche_spulmaschine", "Spülmaschine", "switch", "kuche", &["spuelmaschine"]),
        ent("fan.arc_casual", "Lüfter", "fan", "arbeitszimmer", &["luefter", "fan"]),
        ent("cover.wohnzimmer_rollo", "Rollo Wohnzimmer", "cover", "wohnzimmer", &["rollo wohnzimmer", "rollo"]),
        ent("lock.wohnungstuer", "Wohnungstür", "lock", "flur", &["wohnungstuer", "tuer", "front door", "front"]),
        ent("scene.filmabend", "Filmabend", "scene", "wohnzimmer", &["filmabend", "movie night"]),
        ent("calendar.home", "Kalender", "calendar", "wohnung", &["calendar", "kalender", "termin"]),
    ];
    HomeGraph { entities, areas, ..Default::default() }
}

fn area(id: &str, name: &str, aliases: &[&str]) -> AreaRec {
    AreaRec { area_id: id.to_string(), name: name.to_string(), aliases: aliases.iter().map(|s| s.to_string()).collect(), floor_id: None }
}

fn ent(id: &str, name: &str, domain: &str, area: &str, aliases: &[&str]) -> EntityRec {
    EntityRec {
        entity_id: id.to_string(),
        name: name.to_string(),
        domain: domain.to_string(),
        platform: None,
        area: Some(area.to_string()),
        aliases: aliases.iter().map(|s| s.to_string()).collect(),
        tags: Vec::new(),
    }
}
