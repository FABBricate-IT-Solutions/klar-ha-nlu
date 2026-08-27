//! Hand-written native-script utterances Assist STT actually produces.
use klar_nlu::home::default_home;
use klar_nlu::nlu::parse;
use klar_nlu::session::Session;
use klar_nlu::types::{ParseDecision, Settings};

const CASES: &[(&str, &str, &str)] = &[
    ("ja", "\u{3064}\u{3051}\u{3066} \u{96FB}\u{6C17} \u{30EA}\u{30D3}\u{30F3}\u{30B0}", "HassTurnOn"),
    ("ja", "\u{3064}\u{3051}\u{3066}\u{96FB}\u{6C17}\u{30EA}\u{30D3}\u{30F3}\u{30B0}", "HassTurnOn"),
    ("ja", "\u{6D88}\u{3057}\u{3066} \u{96FB}\u{6C17} \u{30AD}\u{30C3}\u{30C1}\u{30F3}", "HassTurnOff"),
    ("ja", "\u{4F55} \u{4E88}\u{5B9A}", "KlarGetCalendarEvents"),
    ("zh-CN", "\u{6253}\u{5F00} \u{706F} \u{5BA2}\u{5385}", "HassTurnOn"),
    ("zh-CN", "\u{6253}\u{5F00}\u{706F}\u{5BA2}\u{5385}", "HassTurnOn"),
    ("zh-CN", "\u{5173}\u{95ED} \u{706F} \u{53A8}\u{623F}", "HassTurnOff"),
    ("zh-TW", "\u{6253}\u{958B} \u{71C8} \u{5BA2}\u{5EF3}", "HassTurnOn"),
    ("zh-HK", "\u{6253}\u{958B} \u{71C8} \u{5BA2}\u{5EF3}", "HassTurnOn"),
    ("ko", "\u{CF1C} \u{BD88} \u{AC70}\u{C2E4}", "HassTurnOn"),
    ("ko", "\u{CF1C}\u{BD88}\u{AC70}\u{C2E4}", "HassTurnOn"),
    ("ko", "\u{AEBC} \u{BD88} \u{BD80}\u{C5CC}", "HassTurnOff"),
    ("th", "\u{0E40}\u{0E1B}\u{0E34}\u{0E14} \u{0E44}\u{0E1F} \u{0E2B}\u{0E49}\u{0E2D}\u{0E07}\u{0E19}\u{0E31}\u{0E48}\u{0E07}\u{0E40}\u{0E25}\u{0E48}\u{0E19}", "HassTurnOn"),
    ("hi", "\u{091C}\u{0932}\u{093E}\u{0913} \u{092C}\u{0924}\u{094D}\u{0924}\u{0940} \u{092C}\u{0948}\u{0920}\u{0915}", "HassTurnOn"),
    ("hi", "\u{092C}\u{0941}\u{091D}\u{093E}\u{0913} \u{092C}\u{0924}\u{094D}\u{0924}\u{0940} \u{0930}\u{0938}\u{094B}\u{0908}", "HassTurnOff"),
    ("hi", "\u{0915}\u{0948}\u{0932}\u{0947}\u{0902}\u{0921}\u{0930} \u{092E}\u{0947}\u{0902} \u{0915}\u{094D}\u{092F}\u{093E} \u{0939}\u{0948}", "KlarGetCalendarEvents"),
    ("bn", "\u{099C}\u{09CD}\u{09AC}\u{09BE}\u{09B2}\u{09BE}\u{0993} \u{0986}\u{09B2}\u{09CB} \u{09AC}\u{09B8}\u{09BE}\u{09B0}", "HassTurnOn"),
    ("ta", "\u{0B8F}\u{0BB1}\u{0BCD}\u{0BB1}\u{0BC1} \u{0BB5}\u{0BBF}\u{0BB3}\u{0B95}\u{0BCD}\u{0B95}\u{0BC1} \u{0B85}\u{0BB1}\u{0BC8}", "HassTurnOn"),
    ("te", "\u{0C35}\u{0C46}\u{0C32}\u{0C3F}\u{0C17}\u{0C3F}\u{0C02}\u{0C1A}\u{0C41} \u{0C35}\u{0C46}\u{0C32}\u{0C41}\u{0C17}\u{0C41} \u{0C17}\u{0C26}\u{0C3F}", "HassTurnOn"),
    ("kn", "\u{0CB9}\u{0C9A}\u{0CCD}\u{0C9A}\u{0CC1} \u{0CAC}\u{0CC6}\u{0CB3}\u{0C95}\u{0CC1} \u{0CB9}\u{0CBE}\u{0CB2}\u{0CCD}", "HassTurnOn"),
    ("ml", "\u{0D24}\u{0D46}\u{0D33}\u{0D3F} \u{0D35}\u{0D3F}\u{0D33}\u{0D15}\u{0D4D}\u{0D15}\u{0D4D} \u{0D39}\u{0D3E}\u{0D7E}", "HassTurnOn"),
    ("mr", "\u{091A}\u{093E}\u{0932}\u{0942} \u{0926}\u{093F}\u{0935}\u{093E} \u{092C}\u{0948}\u{0920}\u{0915}", "HassTurnOn"),
    ("gu", "\u{0A9A}\u{0ABE}\u{0AB2}\u{0AC1} \u{0AAA}\u{0ACD}\u{0AB0}\u{0A95}\u{0ABE}\u{0AB6} \u{0A93}\u{0AB0}\u{0AA1}\u{0ACB}", "HassTurnOn"),
    ("pa", "\u{0A1A}\u{0A3E}\u{0A32}\u{0A42} \u{0A30}\u{0A4B}\u{0A38}\u{0A3C}\u{0A28}\u{0A40} \u{0A2C}\u{0A48}\u{0A20}\u{0A15}", "HassTurnOn"),
    ("ne", "\u{092C}\u{093E}\u{0932} \u{092C}\u{0924}\u{094D}\u{0924}\u{0940} \u{092C}\u{0948}\u{0920}\u{0915}", "HassTurnOn"),
    ("hy", "\u{0574}\u{056B}\u{0561}\u{0581}\u{0580}\u{0578}\u{0582} \u{056C}\u{0578}\u{0582}\u{0575}\u{057D} \u{0570}\u{0575}\u{0578}\u{0582}\u{0580}\u{0561}\u{057D}\u{0565}\u{0576}\u{0575}\u{0561}\u{056F}", "HassTurnOn"),
    ("ka", "\u{10E9}\u{10D0}\u{10E0}\u{10D7}\u{10D4} \u{10E1}\u{10D8}\u{10DC}\u{10D0}\u{10D7}\u{10DA}\u{10D4} \u{10DB}\u{10D8}\u{10E1}\u{10D0}\u{10E6}\u{10D4}\u{10D1}\u{10D8}", "HassTurnOn"),
    ("mn", "\u{0430}\u{0441}\u{0430}\u{0430} \u{0433}\u{044D}\u{0440}\u{044D}\u{043B} \u{0437}\u{043E}\u{0447}\u{043D}\u{044B}", "HassTurnOn"),
    ("ar", "\u{0634}\u{063A}\u{0644} \u{0636}\u{0648}\u{0621} \u{0635}\u{0627}\u{0644}\u{0648}\u{0646}", "HassTurnOn"),
    ("he", "\u{05D4}\u{05D3}\u{05DC}\u{05E7} \u{05D0}\u{05D5}\u{05E8} \u{05E1}\u{05DC}\u{05D5}\u{05DF}", "HassTurnOn"),
    ("fa", "\u{0631}\u{0648}\u{0634}\u{0646} \u{0686}\u{0631}\u{0627}\u{063A} \u{067E}\u{0630}\u{06CC}\u{0631}\u{0627}\u{06CC}\u{06CC}", "HassTurnOn"),
    ("ur", "\u{0686}\u{0627}\u{0644}\u{0648} \u{0631}\u{0648}\u{0634}\u{0646}\u{06CC} \u{0644}\u{0627}\u{0646}\u{062C}", "HassTurnOn"),
    ("bg", "\u{0432}\u{043A}\u{043B}\u{044E}\u{0447}\u{0438} \u{0441}\u{0432}\u{0435}\u{0442}\u{043B}\u{0438}\u{043D}\u{0430} \u{0445}\u{043E}\u{043B}", "HassTurnOn"),
    ("el", "\u{03B1}\u{03BD}\u{03B1}\u{03C8}\u{03B5} \u{03C6}\u{03C9}\u{03C2} \u{03C3}\u{03B1}\u{03BB}\u{03BF}\u{03BD}\u{03B9}", "HassTurnOn"),
    ("uk", "\u{0443}\u{0432}\u{0456}\u{043C}\u{043A}\u{043D}\u{0438} \u{0441}\u{0432}\u{0456}\u{0442}\u{043B}\u{043E} \u{0432}\u{0456}\u{0442}\u{0430}\u{043B}\u{044C}\u{043D}\u{044F}", "HassTurnOn"),
    ("sr", "\u{0443}\u{043A}\u{0459}\u{0443}\u{0447}\u{0438} \u{0441}\u{0432}\u{0435}\u{0442}\u{043B}\u{043E} \u{0434}\u{043D}\u{0435}\u{0432}\u{043D}\u{0430}", "HassTurnOn"),
    ("de", "mach das licht im wohnzimmer an", "HassTurnOn"),
    ("en", "turn on the living room light", "HassTurnOn"),
    ("fr", "allume la lumiere du salon", "HassTurnOn"),
    ("nl", "zet het licht in de woonkamer aan", "HassTurnOn"),
    ("es", "enciende la luz del salon", "HassTurnOn"),
    ("it", "accendi la luce in soggiorno", "HassTurnOn"),
    ("pt", "liga a luz da sala", "HassTurnOn"),
    ("pl", "wlacz swiatlo w salon", "HassTurnOn"),
    ("tr", "yak isik salon", "HassTurnOn"),
    ("ca", "encen la llum de la sala", "HassTurnOn"),
    ("ro", "aprinde lumina din sufragerie", "HassTurnOn"),
    ("da", "taend lys i stue", "HassTurnOn"),
    ("nb", "skru lys i stue", "HassTurnOn"),
    ("sv", "tand ljus i vardagsrum", "HassTurnOn"),
    ("fi", "sytyta valo olohuone", "HassTurnOn"),
    ("de-CH", "mach s liecht im wohnzimmer", "HassTurnOn"),
    ("de-AT", "mach das licht im wohnzimmer an", "HassTurnOn"),
    ("en-GB", "turn on the lounge light", "HassTurnOn"),
    ("pt-BR", "liga a luz da sala", "HassTurnOn"),
    ("af", "skakel die sitkamer lig", "HassTurnOn"),
    ("cs", "zapni svetlo v obyvak", "HassTurnOn"),
    ("sk", "zapni svetlo v obyvacka", "HassTurnOn"),
    ("hu", "kapcsold a villany nappali", "HassTurnOn"),
    ("hr", "upali svjetlo u dnevni", "HassTurnOn"),
    ("sl", "prizgi luc v dnevna", "HassTurnOn"),
    ("sr-Latn", "ukljuci svetlo u dnevna", "HassTurnOn"),
    ("cy", "tro y golau yn yr ystafell", "HassTurnOn"),
    ("et", "lulita vali elutuba", "HassTurnOn"),
    ("eu", "piztu argia egongela", "HassTurnOn"),
    ("ga", "cas an solas sa seomra", "HassTurnOn"),
    ("gl", "acende a luz do salon", "HassTurnOn"),
    ("is", "kveiktu ljos i stofa", "HassTurnOn"),
    ("lb", "maach d luucht am wunnen", "HassTurnOn"),
    ("kw", "enow an golow y'n stafell", "HassTurnOn"),
    ("lt", "ijunk sviesa svetaine", "HassTurnOn"),
    ("lv", "iesledz gaisma istaba", "HassTurnOn"),
    ("id", "nyalakan lampu ruang", "HassTurnOn"),
    ("ms", "hidupkan lampu ruang", "HassTurnOn"),
    ("sw", "washa taa sebuleni", "HassTurnOn"),
    ("vi", "bat bongden phongkhach", "HassTurnOn"),
    ("ja", "tsukete raito ribingu", "HassTurnOn"),
    ("hi", "jalao batti baithak", "HassTurnOn"),
];

#[test]
fn native_script_home_commands() {
    let home = default_home();
    for (lang, text, intent) in CASES {
        let settings = Settings {
            languages: vec![(*lang).into()],
            ..Settings::default()
        };
        let mut session = Session::default();
        let outcome = parse(text, &home, &mut session, &[], &settings);
        let names: Vec<_> = outcome
            .plan
            .as_ref()
            .map(|plan| plan.intents().into_iter().map(|item| item.name).collect())
            .unwrap_or_default();
        assert!(
            matches!(outcome.decision, ParseDecision::Execute) && names.iter().any(|name| name == intent),
            "{lang} {text} decision={:?} intents={names:?}",
            outcome.decision
        );
    }
}

#[test]
fn every_compiled_pack_has_a_handwritten_utterance() {
    let covered: std::collections::HashSet<_> = CASES.iter().map(|(lang, _, _)| *lang).collect();
    for id in klar_nlu::lang::LangId::all() {
        assert!(covered.contains(id.code()), "missing handwritten utterance for {}", id.code());
    }
}
