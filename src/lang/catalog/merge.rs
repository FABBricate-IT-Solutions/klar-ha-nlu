use super::Catalog;
use crate::lang::groups::GroupClarify;
use crate::lang::groups::LanguagePack as Pack;
use crate::lang::morphology::Morphology;
use std::collections::{HashMap, HashSet};

macro_rules! extend_sets {
    ($dst:expr, $src:expr; $($field:ident),+ $(,)?) => {
        $($dst.$field.extend($src.$field.iter().copied());)+
    };
}

impl Catalog {
    pub(in crate::lang) fn merge(packs: &[&'static Pack]) -> Self {
        let mut c = Self::empty();
        c.packs = packs.to_vec();
        for p in packs {
            c.langs.push(p.id);
            c.speech.push(&p.speech);
            for &(w, k) in p.verbs {
                c.verbs.insert(w, k);
            }
            for &(w, d) in p.maps.domain_map {
                c.domain_map.insert(w, d);
            }
            for &(w, color) in p.maps.colors {
                c.colors.insert(w, color);
            }
            for &(w, n) in p.maps.numbers {
                c.numbers.insert(w, n);
            }
            c.number_styles.push(p.maps.number_style);
            for &(w, aliases) in p.fixtures.fixture_aliases {
                c.fixture_aliases.insert(w, aliases);
            }
            if let Some(g) = &p.fixtures.group_clarify {
                c.group_clarify.push(GroupClarify { trigger: g.trigger, pairs: g.pairs, triples: g.triples });
            }
            c.strip_pairs.extend(p.cues.strip_pairs.iter().copied());
            c.keep_after.extend(p.cues.keep_after.iter().copied());
            c.synonym_pairs.extend(p.cues.synonym_pairs.iter().copied());
            c.scene_synonyms.extend(p.cues.scene_synonyms.iter().copied());
            extend_sets!(c, p.talk; fillers, action_keep, conjunctions, particles, affirm, or_words, all_words, query_hint, question_starts, question_words, correction, correction_phrases, clarify_pick);
            extend_sets!(c, p.nouns; light_nouns, light_singular, light_plural, cover_nouns, curtain_nouns, fan_nouns, climate_nouns, media_nouns, lock_nouns, door_nouns, garage_words, garage_cover, timer_nouns, list_nouns, vacuum_nouns, scene_nouns, script_words, switch_plural, device_side, named_device);
            extend_sets!(c, p.fixtures; island, ceiling, lamp_fixture, pendant, bedside, left, right, sides, singular_lamp, singular_lamp_block);
            extend_sets!(c, p.cues; power_words, command_hedges, skip_light, laundry_area, laundry_machines, kitchen, open_words, close_words, roll_close, unlock_follow, cover_open_follow, garage_lock_block, on_words, off_words, scene_named, temp_query, timer_query, brightness, start_words, replay_on_off, replay_off, sensor_words, lock_verbs, entry_words, oven, laundry_timer, illuminate, list_down, chores, weak_scene, timer_cancel, timer_pause, timer_add, list_complete, playback_resume, vacuum_start, hours, minutes, seconds, list_skip, shopping_names, status_words, window_words, open_close, laundry_hint, bare_switch, outlet_words, tv_words, climate_cool, climate_heat, role_light, role_climate, role_media, role_fan, generic, room_level, extra_device_nouns, article_one);
            extend_sets!(c, p.maps; room_index_nouns);
            c.chat_greet.extend(p.chat.greet.iter().copied());
            c.chat_thanks.extend(p.chat.thanks.iter().copied());
            c.chat_feeling.extend(p.chat.feeling.iter().copied());
            c.chat_identity.extend(p.chat.identity.iter().copied());
            c.chat_tell.extend(p.chat.tell.iter().copied());
            c.chat_yarn.extend(p.chat.yarn.iter().copied());
            c.chat_world.extend(p.chat.world.iter().copied());
            c.chat_advice.extend(p.chat.advice.iter().copied());
            c.chat_open.extend(p.chat.open.iter().copied());
            c.chat_news.extend(p.chat.news.iter().copied());
            c.chat_news_dismiss.extend(p.chat.news_dismiss.iter().copied());
            if c.news_intro.is_empty() {
                c.news_intro = p.chat.news_intro;
                c.news_nudge = p.chat.news_nudge;
                c.news_done = p.chat.news_done;
            }
            c.morphology.room_suffixes.extend(p.morphology.room_suffixes.iter().copied());
            c.morphology.color_suffixes.extend(p.morphology.color_suffixes.iter().copied());
            c.morphology.linking.extend(p.morphology.linking.iter().copied());
        }
        c
    }

    fn empty() -> Self {
        Self {
            packs: Vec::new(),
            langs: Vec::new(),
            verbs: HashMap::new(),
            domain_map: HashMap::new(),
            colors: HashMap::new(),
            numbers: HashMap::new(),
            number_styles: Vec::new(),
            fixture_aliases: HashMap::new(),
            group_clarify: Vec::new(),
            strip_pairs: Vec::new(),
            keep_after: Vec::new(),
            synonym_pairs: Vec::new(),
            scene_synonyms: Vec::new(),
            speech: Vec::new(),
            fillers: HashSet::new(),
            action_keep: HashSet::new(),
            conjunctions: HashSet::new(),
            particles: HashSet::new(),
            affirm: HashSet::new(),
            or_words: HashSet::new(),
            all_words: HashSet::new(),
            query_hint: HashSet::new(),
            question_starts: HashSet::new(),
            question_words: HashSet::new(),
            correction: HashSet::new(),
            correction_phrases: HashSet::new(),
            clarify_pick: HashSet::new(),
            light_nouns: HashSet::new(),
            light_singular: HashSet::new(),
            light_plural: HashSet::new(),
            cover_nouns: HashSet::new(),
            curtain_nouns: HashSet::new(),
            fan_nouns: HashSet::new(),
            climate_nouns: HashSet::new(),
            media_nouns: HashSet::new(),
            lock_nouns: HashSet::new(),
            door_nouns: HashSet::new(),
            garage_words: HashSet::new(),
            garage_cover: HashSet::new(),
            timer_nouns: HashSet::new(),
            list_nouns: HashSet::new(),
            vacuum_nouns: HashSet::new(),
            scene_nouns: HashSet::new(),
            script_words: HashSet::new(),
            switch_plural: HashSet::new(),
            device_side: HashSet::new(),
            named_device: HashSet::new(),
            island: HashSet::new(),
            ceiling: HashSet::new(),
            lamp_fixture: HashSet::new(),
            pendant: HashSet::new(),
            bedside: HashSet::new(),
            left: HashSet::new(),
            right: HashSet::new(),
            sides: HashSet::new(),
            singular_lamp: HashSet::new(),
            singular_lamp_block: HashSet::new(),
            power_words: HashSet::new(),
            command_hedges: HashSet::new(),
            skip_light: HashSet::new(),
            laundry_area: HashSet::new(),
            laundry_machines: HashSet::new(),
            kitchen: HashSet::new(),
            open_words: HashSet::new(),
            close_words: HashSet::new(),
            roll_close: HashSet::new(),
            unlock_follow: HashSet::new(),
            cover_open_follow: HashSet::new(),
            garage_lock_block: HashSet::new(),
            on_words: HashSet::new(),
            off_words: HashSet::new(),
            scene_named: HashSet::new(),
            temp_query: HashSet::new(),
            timer_query: HashSet::new(),
            brightness: HashSet::new(),
            start_words: HashSet::new(),
            replay_on_off: HashSet::new(),
            replay_off: HashSet::new(),
            sensor_words: HashSet::new(),
            lock_verbs: HashSet::new(),
            entry_words: HashSet::new(),
            oven: HashSet::new(),
            laundry_timer: HashSet::new(),
            illuminate: HashSet::new(),
            list_down: HashSet::new(),
            chores: HashSet::new(),
            weak_scene: HashSet::new(),
            timer_cancel: HashSet::new(),
            timer_pause: HashSet::new(),
            timer_add: HashSet::new(),
            list_complete: HashSet::new(),
            playback_resume: HashSet::new(),
            vacuum_start: HashSet::new(),
            hours: HashSet::new(),
            minutes: HashSet::new(),
            seconds: HashSet::new(),
            list_skip: HashSet::new(),
            shopping_names: HashSet::new(),
            status_words: HashSet::new(),
            window_words: HashSet::new(),
            open_close: HashSet::new(),
            laundry_hint: HashSet::new(),
            bare_switch: HashSet::new(),
            outlet_words: HashSet::new(),
            tv_words: HashSet::new(),
            climate_cool: HashSet::new(),
            climate_heat: HashSet::new(),
            role_light: HashSet::new(),
            role_climate: HashSet::new(),
            role_media: HashSet::new(),
            role_fan: HashSet::new(),
            generic: HashSet::new(),
            room_level: HashSet::new(),
            extra_device_nouns: HashSet::new(),
            article_one: HashSet::new(),
            room_index_nouns: HashSet::new(),
            chat_greet: HashSet::new(),
            chat_thanks: HashSet::new(),
            chat_feeling: HashSet::new(),
            chat_identity: HashSet::new(),
            chat_tell: HashSet::new(),
            chat_yarn: HashSet::new(),
            chat_world: HashSet::new(),
            chat_advice: HashSet::new(),
            chat_open: HashSet::new(),
            chat_news: HashSet::new(),
            chat_news_dismiss: HashSet::new(),
            news_intro: "",
            news_nudge: "",
            news_done: "",
            morphology: Morphology::default(),
            pack_intents: Vec::new(),
        }
    }
}
