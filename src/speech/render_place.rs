//! Locale tables for empty-place lines, colors, and HA state words.

use crate::types::SpeechSnapshot;

const COLORS: &[(&str, &str, &str)] = &[
    ("red", "rot", "red"),
    ("blue", "blau", "blue"),
    ("green", "grün", "green"),
    ("yellow", "gelb", "yellow"),
    ("orange", "orange", "orange"),
    ("pink", "pink", "pink"),
    ("black", "schwarz", "black"),
    ("white", "weiß", "white"),
    ("warmwhite", "warmweiß", "warm white"),
    ("purple", "lila", "purple"),
];

const EMPTY_PLACE: &[(&str, &str)] = &[
    ("de", "Keine Geräte."),
    ("en", "No devices."),
    ("fr", "Aucun appareil."),
    ("nl", "Geen apparaten."),
    ("es", "Ningún aparato."),
    ("it", "Nessun dispositivo."),
    ("pt", "Nenhum aparelho."),
    ("ca", "Cap aparell."),
    ("ro", "Niciun aparat."),
    ("da", "Ingen enheder."),
    ("nb", "Ingen enheter."),
    ("sv", "Inga enheter."),
    ("fi", "Ei laitteita."),
    ("af", "Geen toestelle."),
    ("cs", "Žádná zařízení."),
    ("sk", "Žiadne zariadenia."),
    ("pl", "Brak urządzeń."),
    ("hu", "Nincs eszköz."),
    ("hr", "Nema uređaja."),
    ("sl", "Ni naprav."),
    ("bg", "Няма устройства."),
    ("el", "Κανένα συσκευή."),
    ("sr", "Нема уређаја."),
    ("uk", "Немає пристроїв."),
    ("zh-CN", "没有设备。"),
    ("zh-TW", "沒有裝置。"),
    ("zh-HK", "冇裝置。"),
    ("ar", "لا أجهزة."),
    ("he", "אין מכשירים."),
    ("fa", "دستگاهی نیست."),
    ("ur", "کوئی آلہ نہیں."),
    ("tr", "Cihaz yok."),
    ("th", "ไม่มีอุปกรณ์"),
    ("ko", "기기 없음."),
    ("ja", "機器はありません。"),
    ("cy", "Dim dyfeisiau."),
    ("et", "Seadmeid pole."),
    ("eu", "Ez dago gailurik."),
    ("ga", "Níl aon ghléas."),
    ("gl", "Ningún aparello."),
    ("is", "Engin tæki."),
    ("lb", "Keng Geräter."),
    ("kw", "Ny vyjy."),
    ("lt", "Nėra įrenginių."),
    ("lv", "Nav ierīču."),
    ("id", "Tidak ada perangkat."),
    ("ms", "Tiada peranti."),
    ("sw", "Hakuna vifaa."),
    ("vi", "Không có thiết bị."),
    ("hi", "कोई उपकरण नहीं."),
    ("bn", "কোনো যন্ত্র নেই."),
    ("gu", "કોઈ ઉપકરણ નથી."),
    ("kn", "ಯಾವುದೇ ಸಾಧನವಿಲ್ಲ."),
    ("ml", "ഉപകരണങ്ങളില്ല."),
    ("mr", "साधने नाहीत."),
    ("ta", "சாதனங்கள் இல்லை."),
    ("te", "పరికరాలు లేవు."),
    ("pa", "ਕੋਈ ਯੰਤਰ ਨਹੀਂ."),
    ("ne", "कुनै उपकरण छैन."),
    ("hy", "Սարքեր չկան."),
    ("ka", "მოწყობილობა არ არის."),
    ("mn", "Төхөөрөмж байхгүй."),
    ("sr-Latn", "Nema uređaja."),
    ("pt-BR", "Nenhum aparelho."),
    ("en-GB", "No devices."),
    ("de-CH", "Kei Grät."),
    ("de-AT", "Keine Geräte."),
];

const DE_STATE: &[(&str, &str)] = &[
    ("on", "an"),
    ("off", "aus"),
    ("unavailable", "nicht da"),
    ("unknown", "unbekannt"),
    ("open", "offen"),
    ("closed", "zu"),
    ("locked", "zu"),
    ("unlocked", "offen"),
    ("playing", "spielt"),
    ("paused", "pausiert"),
    ("idle", "bereit"),
    ("heat", "heizt"),
    ("cool", "kühlt"),
    ("cloudy", "bewölkt"),
    ("partlycloudy", "teilweise bewölkt"),
    ("rainy", "regnerisch"),
    ("sunny", "sonnig"),
    ("clear", "klar"),
];

pub(super) fn empty_place(pack: &str) -> String {
    let exact = EMPTY_PLACE.iter().find(|(code, _)| *code == pack);
    if let Some((_, line)) = exact {
        return (*line).to_string();
    }
    let base = pack.split('-').next().unwrap_or(pack);
    EMPTY_PLACE.iter().find(|(code, _)| *code == base).map(|(_, line)| (*line).to_string()).unwrap_or_else(|| "No devices.".into())
}

pub(super) fn speak_state(raw: &str, pack: &str) -> String {
    let base = pack.split('-').next().unwrap_or(pack);
    if base == "de" {
        return DE_STATE
            .iter()
            .find(|(key, _)| *key == raw)
            .map(|(_, spoken)| (*spoken).to_string())
            .unwrap_or_else(|| raw.replace('.', ","));
    }
    if raw == "off" {
        return match base {
            "fr" => "éteinte".into(),
            "nl" => "uit".into(),
            _ => "off".into(),
        };
    }
    if raw == "on" {
        return match base {
            "fr" => "allumée".into(),
            "nl" => "aan".into(),
            _ => "on".into(),
        };
    }
    raw.to_string()
}

pub(super) fn color_word(raw: Option<&str>, de: bool) -> Option<String> {
    let color = raw?;
    COLORS
        .iter()
        .find(|(key, _, _)| *key == color)
        .map(|(_, german, english)| if de { (*german).to_string() } else { (*english).to_string() })
}

pub(super) fn slot<'a>(snap: &'a SpeechSnapshot, name: &str) -> Option<&'a str> {
    snap.intent.slots.iter().find(|slot| slot.name == name).map(|slot| slot.value.as_str()).filter(|value| !value.is_empty())
}
