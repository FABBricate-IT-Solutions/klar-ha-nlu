use super::words::WordKey;
use super::Catalog;
use crate::lang::groups::GroupClarify;
use crate::lang::groups::LanguagePack as Pack;
use crate::lang::morphology::Morphology;
use std::collections::HashMap;

type WordPick = fn(&Pack) -> &'static [&'static str];

const WORD_SOURCES: &[(WordKey, WordPick)] = &[
    (WordKey::Fillers, |p: &Pack| p.talk.fillers),
    (WordKey::ActionKeep, |p: &Pack| p.talk.action_keep),
    (WordKey::Conjunctions, |p: &Pack| p.talk.conjunctions),
    (WordKey::Particles, |p: &Pack| p.talk.particles),
    (WordKey::Affirm, |p: &Pack| p.talk.affirm),
    (WordKey::OrWords, |p: &Pack| p.talk.or_words),
    (WordKey::AllWords, |p: &Pack| p.talk.all_words),
    (WordKey::QueryHint, |p: &Pack| p.talk.query_hint),
    (WordKey::QuestionStarts, |p: &Pack| p.talk.question_starts),
    (WordKey::QuestionWords, |p: &Pack| p.talk.question_words),
    (WordKey::Correction, |p: &Pack| p.talk.correction),
    (WordKey::CorrectionPhrases, |p: &Pack| p.talk.correction_phrases),
    (WordKey::ClarifyPick, |p: &Pack| p.talk.clarify_pick),
    (WordKey::LightNouns, |p: &Pack| p.nouns.light_nouns),
    (WordKey::LightSingular, |p: &Pack| p.nouns.light_singular),
    (WordKey::LightPlural, |p: &Pack| p.nouns.light_plural),
    (WordKey::CoverNouns, |p: &Pack| p.nouns.cover_nouns),
    (WordKey::CurtainNouns, |p: &Pack| p.nouns.curtain_nouns),
    (WordKey::FanNouns, |p: &Pack| p.nouns.fan_nouns),
    (WordKey::ClimateNouns, |p: &Pack| p.nouns.climate_nouns),
    (WordKey::MediaNouns, |p: &Pack| p.nouns.media_nouns),
    (WordKey::LockNouns, |p: &Pack| p.nouns.lock_nouns),
    (WordKey::DoorNouns, |p: &Pack| p.nouns.door_nouns),
    (WordKey::GarageWords, |p: &Pack| p.nouns.garage_words),
    (WordKey::GarageCover, |p: &Pack| p.nouns.garage_cover),
    (WordKey::TimerNouns, |p: &Pack| p.nouns.timer_nouns),
    (WordKey::ListNouns, |p: &Pack| p.nouns.list_nouns),
    (WordKey::VacuumNouns, |p: &Pack| p.nouns.vacuum_nouns),
    (WordKey::SceneNouns, |p: &Pack| p.nouns.scene_nouns),
    (WordKey::ScriptWords, |p: &Pack| p.nouns.script_words),
    (WordKey::SwitchPlural, |p: &Pack| p.nouns.switch_plural),
    (WordKey::DeviceSide, |p: &Pack| p.nouns.device_side),
    (WordKey::NamedDevice, |p: &Pack| p.nouns.named_device),
    (WordKey::Island, |p: &Pack| p.fixtures.island),
    (WordKey::Ceiling, |p: &Pack| p.fixtures.ceiling),
    (WordKey::LampFixture, |p: &Pack| p.fixtures.lamp_fixture),
    (WordKey::Pendant, |p: &Pack| p.fixtures.pendant),
    (WordKey::Bedside, |p: &Pack| p.fixtures.bedside),
    (WordKey::Left, |p: &Pack| p.fixtures.left),
    (WordKey::Right, |p: &Pack| p.fixtures.right),
    (WordKey::Sides, |p: &Pack| p.fixtures.sides),
    (WordKey::SingularLamp, |p: &Pack| p.fixtures.singular_lamp),
    (WordKey::SingularLampBlock, |p: &Pack| p.fixtures.singular_lamp_block),
    (WordKey::PowerWords, |p: &Pack| p.cues.power_words),
    (WordKey::CommandHedges, |p: &Pack| p.cues.command_hedges),
    (WordKey::SkipLight, |p: &Pack| p.cues.skip_light),
    (WordKey::LaundryArea, |p: &Pack| p.cues.laundry_area),
    (WordKey::LaundryMachines, |p: &Pack| p.cues.laundry_machines),
    (WordKey::Kitchen, |p: &Pack| p.cues.kitchen),
    (WordKey::OpenWords, |p: &Pack| p.cues.open_words),
    (WordKey::CloseWords, |p: &Pack| p.cues.close_words),
    (WordKey::RollClose, |p: &Pack| p.cues.roll_close),
    (WordKey::UnlockFollow, |p: &Pack| p.cues.unlock_follow),
    (WordKey::CoverOpenFollow, |p: &Pack| p.cues.cover_open_follow),
    (WordKey::GarageLockBlock, |p: &Pack| p.cues.garage_lock_block),
    (WordKey::OnWords, |p: &Pack| p.cues.on_words),
    (WordKey::OffWords, |p: &Pack| p.cues.off_words),
    (WordKey::SceneNamed, |p: &Pack| p.cues.scene_named),
    (WordKey::TempQuery, |p: &Pack| p.cues.temp_query),
    (WordKey::TimerQuery, |p: &Pack| p.cues.timer_query),
    (WordKey::Brightness, |p: &Pack| p.cues.brightness),
    (WordKey::StartWords, |p: &Pack| p.cues.start_words),
    (WordKey::ReplayOnOff, |p: &Pack| p.cues.replay_on_off),
    (WordKey::ReplayOff, |p: &Pack| p.cues.replay_off),
    (WordKey::SensorWords, |p: &Pack| p.cues.sensor_words),
    (WordKey::LockVerbs, |p: &Pack| p.cues.lock_verbs),
    (WordKey::EntryWords, |p: &Pack| p.cues.entry_words),
    (WordKey::Oven, |p: &Pack| p.cues.oven),
    (WordKey::LaundryTimer, |p: &Pack| p.cues.laundry_timer),
    (WordKey::Illuminate, |p: &Pack| p.cues.illuminate),
    (WordKey::ListDown, |p: &Pack| p.cues.list_down),
    (WordKey::Chores, |p: &Pack| p.cues.chores),
    (WordKey::WeakScene, |p: &Pack| p.cues.weak_scene),
    (WordKey::TimerCancel, |p: &Pack| p.cues.timer_cancel),
    (WordKey::TimerPause, |p: &Pack| p.cues.timer_pause),
    (WordKey::TimerAdd, |p: &Pack| p.cues.timer_add),
    (WordKey::ListComplete, |p: &Pack| p.cues.list_complete),
    (WordKey::PlaybackResume, |p: &Pack| p.cues.playback_resume),
    (WordKey::VacuumStart, |p: &Pack| p.cues.vacuum_start),
    (WordKey::Hours, |p: &Pack| p.cues.hours),
    (WordKey::Minutes, |p: &Pack| p.cues.minutes),
    (WordKey::Seconds, |p: &Pack| p.cues.seconds),
    (WordKey::ListSkip, |p: &Pack| p.cues.list_skip),
    (WordKey::ShoppingNames, |p: &Pack| p.cues.shopping_names),
    (WordKey::StatusWords, |p: &Pack| p.cues.status_words),
    (WordKey::WindowWords, |p: &Pack| p.cues.window_words),
    (WordKey::OpenClose, |p: &Pack| p.cues.open_close),
    (WordKey::LaundryHint, |p: &Pack| p.cues.laundry_hint),
    (WordKey::BareSwitch, |p: &Pack| p.cues.bare_switch),
    (WordKey::OutletWords, |p: &Pack| p.cues.outlet_words),
    (WordKey::TvWords, |p: &Pack| p.cues.tv_words),
    (WordKey::ClimateCool, |p: &Pack| p.cues.climate_cool),
    (WordKey::ClimateHeat, |p: &Pack| p.cues.climate_heat),
    (WordKey::RoleLight, |p: &Pack| p.cues.role_light),
    (WordKey::RoleClimate, |p: &Pack| p.cues.role_climate),
    (WordKey::RoleMedia, |p: &Pack| p.cues.role_media),
    (WordKey::RoleFan, |p: &Pack| p.cues.role_fan),
    (WordKey::Generic, |p: &Pack| p.cues.generic),
    (WordKey::RoomLevel, |p: &Pack| p.cues.room_level),
    (WordKey::ExtraDeviceNouns, |p: &Pack| p.cues.extra_device_nouns),
    (WordKey::ArticleOne, |p: &Pack| p.cues.article_one),
    (WordKey::RoomIndexNouns, |p: &Pack| p.maps.room_index_nouns),
    (WordKey::ChatGreet, |p: &Pack| p.chat.greet),
    (WordKey::ChatThanks, |p: &Pack| p.chat.thanks),
    (WordKey::ChatFeeling, |p: &Pack| p.chat.feeling),
    (WordKey::ChatIdentity, |p: &Pack| p.chat.identity),
    (WordKey::ChatTell, |p: &Pack| p.chat.tell),
    (WordKey::ChatYarn, |p: &Pack| p.chat.yarn),
    (WordKey::ChatWorld, |p: &Pack| p.chat.world),
    (WordKey::ChatAdvice, |p: &Pack| p.chat.advice),
    (WordKey::ChatOpen, |p: &Pack| p.chat.open),
    (WordKey::ChatNews, |p: &Pack| p.chat.news),
    (WordKey::ChatNewsDismiss, |p: &Pack| p.chat.news_dismiss),
];

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
            for (key, pick) in WORD_SOURCES {
                c.sets.entry(*key).or_default().extend(pick(p).iter().copied());
            }
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
            sets: HashMap::new(),
            news_intro: "",
            news_nudge: "",
            news_done: "",
            morphology: Morphology::default(),
            pack_intents: Vec::new(),
        }
    }
}
