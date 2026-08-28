use super::keys::WordKey;
use super::Catalog;
use std::collections::HashSet;
use std::sync::OnceLock;

fn empty_set() -> &'static HashSet<&'static str> {
    static EMPTY: OnceLock<HashSet<&'static str>> = OnceLock::new();
    EMPTY.get_or_init(HashSet::new)
}

impl Catalog {
    pub fn words(&self, key: WordKey) -> &HashSet<&'static str> {
        self.sets.get(&key).unwrap_or_else(|| empty_set())
    }

    pub fn words_mut(&mut self, key: WordKey) -> &mut HashSet<&'static str> {
        self.sets.entry(key).or_default()
    }

    pub fn fillers(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Fillers)
    }
    pub fn action_keep(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ActionKeep)
    }
    pub fn conjunctions(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Conjunctions)
    }
    pub fn particles(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Particles)
    }
    pub fn affirm(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Affirm)
    }
    pub fn or_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OrWords)
    }
    pub fn all_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::AllWords)
    }
    pub fn query_hint(&self) -> &HashSet<&'static str> {
        self.words(WordKey::QueryHint)
    }
    pub fn question_starts(&self) -> &HashSet<&'static str> {
        self.words(WordKey::QuestionStarts)
    }
    pub fn question_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::QuestionWords)
    }
    pub fn correction(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Correction)
    }
    pub fn correction_phrases(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CorrectionPhrases)
    }
    pub fn clarify_pick(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ClarifyPick)
    }
    pub fn light_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LightNouns)
    }
    pub fn light_singular(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LightSingular)
    }
    pub fn light_plural(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LightPlural)
    }
    pub fn cover_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CoverNouns)
    }
    pub fn curtain_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CurtainNouns)
    }
    pub fn fan_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::FanNouns)
    }
    pub fn climate_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ClimateNouns)
    }
    pub fn media_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::MediaNouns)
    }
    pub fn lock_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LockNouns)
    }
    pub fn door_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::DoorNouns)
    }
    pub fn garage_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::GarageWords)
    }
    pub fn garage_cover(&self) -> &HashSet<&'static str> {
        self.words(WordKey::GarageCover)
    }
    pub fn timer_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TimerNouns)
    }
    pub fn list_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ListNouns)
    }
    pub fn calendar_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarNouns)
    }
    pub fn calendar_query(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarQuery)
    }
    pub fn calendar_create(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarCreate)
    }
    pub fn calendar_today(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarToday)
    }
    pub fn calendar_tomorrow(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarTomorrow)
    }
    pub fn calendar_when(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarWhen)
    }
    pub fn calendar_delete(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarDelete)
    }
    pub fn calendar_move(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CalendarMove)
    }
    pub fn vacuum_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::VacuumNouns)
    }
    pub fn scene_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SceneNouns)
    }
    pub fn script_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ScriptWords)
    }
    pub fn switch_plural(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SwitchPlural)
    }
    pub fn device_side(&self) -> &HashSet<&'static str> {
        self.words(WordKey::DeviceSide)
    }
    pub fn named_device(&self) -> &HashSet<&'static str> {
        self.words(WordKey::NamedDevice)
    }
    pub fn island(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Island)
    }
    pub fn ceiling(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Ceiling)
    }
    pub fn lamp_fixture(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LampFixture)
    }
    pub fn pendant(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Pendant)
    }
    pub fn bedside(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Bedside)
    }
    pub fn left(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Left)
    }
    pub fn right(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Right)
    }
    pub fn sides(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Sides)
    }
    pub fn singular_lamp(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SingularLamp)
    }
    pub fn singular_lamp_block(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SingularLampBlock)
    }
    pub fn power_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::PowerWords)
    }
    pub fn command_hedges(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CommandHedges)
    }
    pub fn skip_light(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SkipLight)
    }
    pub fn laundry_area(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LaundryArea)
    }
    pub fn laundry_machines(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LaundryMachines)
    }
    pub fn kitchen(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Kitchen)
    }
    pub fn open_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OpenWords)
    }
    pub fn close_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CloseWords)
    }
    pub fn roll_close(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RollClose)
    }
    pub fn unlock_follow(&self) -> &HashSet<&'static str> {
        self.words(WordKey::UnlockFollow)
    }
    pub fn cover_open_follow(&self) -> &HashSet<&'static str> {
        self.words(WordKey::CoverOpenFollow)
    }
    pub fn garage_lock_block(&self) -> &HashSet<&'static str> {
        self.words(WordKey::GarageLockBlock)
    }
    pub fn on_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OnWords)
    }
    pub fn off_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OffWords)
    }
    pub fn scene_named(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SceneNamed)
    }
    pub fn temp_query(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TempQuery)
    }
    pub fn timer_query(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TimerQuery)
    }
    pub fn brightness(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Brightness)
    }
    pub fn start_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::StartWords)
    }
    pub fn replay_on_off(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ReplayOnOff)
    }
    pub fn replay_off(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ReplayOff)
    }
    pub fn sensor_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::SensorWords)
    }
    pub fn lock_verbs(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LockVerbs)
    }
    pub fn entry_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::EntryWords)
    }
    pub fn oven(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Oven)
    }
    pub fn laundry_timer(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LaundryTimer)
    }
    pub fn illuminate(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Illuminate)
    }
    pub fn list_down(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ListDown)
    }
    pub fn chores(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Chores)
    }
    pub fn weak_scene(&self) -> &HashSet<&'static str> {
        self.words(WordKey::WeakScene)
    }
    pub fn timer_cancel(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TimerCancel)
    }
    pub fn timer_pause(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TimerPause)
    }
    pub fn timer_add(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TimerAdd)
    }
    pub fn list_complete(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ListComplete)
    }
    pub fn playback_resume(&self) -> &HashSet<&'static str> {
        self.words(WordKey::PlaybackResume)
    }
    pub fn vacuum_start(&self) -> &HashSet<&'static str> {
        self.words(WordKey::VacuumStart)
    }
    pub fn hours(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Hours)
    }
    pub fn minutes(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Minutes)
    }
    pub fn seconds(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Seconds)
    }
    pub fn list_skip(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ListSkip)
    }
    pub fn shopping_names(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ShoppingNames)
    }
    pub fn status_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::StatusWords)
    }
    pub fn window_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::WindowWords)
    }
    pub fn open_close(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OpenClose)
    }
    pub fn laundry_hint(&self) -> &HashSet<&'static str> {
        self.words(WordKey::LaundryHint)
    }
    pub fn bare_switch(&self) -> &HashSet<&'static str> {
        self.words(WordKey::BareSwitch)
    }
    pub fn outlet_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::OutletWords)
    }
    pub fn tv_words(&self) -> &HashSet<&'static str> {
        self.words(WordKey::TvWords)
    }
    pub fn climate_cool(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ClimateCool)
    }
    pub fn climate_heat(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ClimateHeat)
    }
    pub fn role_light(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoleLight)
    }
    pub fn role_climate(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoleClimate)
    }
    pub fn role_media(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoleMedia)
    }
    pub fn role_fan(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoleFan)
    }
    pub fn generic(&self) -> &HashSet<&'static str> {
        self.words(WordKey::Generic)
    }
    pub fn room_level(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoomLevel)
    }
    pub fn extra_device_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ExtraDeviceNouns)
    }
    pub fn article_one(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ArticleOne)
    }
    pub fn room_index_nouns(&self) -> &HashSet<&'static str> {
        self.words(WordKey::RoomIndexNouns)
    }
    pub fn chat_greet(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatGreet)
    }
    pub fn chat_thanks(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatThanks)
    }
    pub fn chat_feeling(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatFeeling)
    }
    pub fn chat_identity(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatIdentity)
    }
    pub fn chat_tell(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatTell)
    }
    pub fn chat_yarn(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatYarn)
    }
    pub fn chat_world(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatWorld)
    }
    pub fn chat_advice(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatAdvice)
    }
    pub fn chat_open(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatOpen)
    }
    pub fn chat_news(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatNews)
    }
    pub fn chat_news_dismiss(&self) -> &HashSet<&'static str> {
        self.words(WordKey::ChatNewsDismiss)
    }
}
