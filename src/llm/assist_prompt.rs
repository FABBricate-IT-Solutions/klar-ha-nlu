//! Assist system prompts. Personality is applied here; HA must not prepend refine_prompt.

use super::assist_rag::rag_prompt;
use super::assist_yarn::{yarn_body, yarn_request};
use super::refine_prompt::language_lock;

const CHAT_ONLY: &[(&str, &str)] = &[
    ("de", "Du antwortest nur im Gespräch. Steuere keine Geräte und rufe keine Home-Assistant-Werkzeuge auf."),
    ("en", "Reply in conversation only. Do not control devices and do not call Home Assistant tools."),
    ("ja", "会話だけで答えてください。機器は操作せず、Home Assistantのツールも呼ばないでください。"),
    ("zh-CN", "只对话回答。不要控制设备，也不要调用 Home Assistant 工具。"),
    ("zh-TW", "只對話回答。不要控制裝置，也不要呼叫 Home Assistant 工具。"),
    ("zh-HK", "只對話答。唔好控制裝置，亦唔好叫 Home Assistant 工具。"),
    ("ko", "대화로만 답하세요. 기기를 제어하지 말고 Home Assistant 도구도 호출하지 마세요."),
    ("hi", "केवल बातचीत में जवाब दो। उपकरण मत चलाओ और Home Assistant औज़ार मत बुलाओ।"),
    ("ar", "أجب في المحادثة فقط. لا تتحكم في الأجهزة ولا تستدع أدوات Home Assistant."),
    ("th", "ตอบเฉพาะบทสนทนา อย่าควบคุมอุปกรณ์ และอย่าเรียกเครื่องมือ Home Assistant"),
];

const TOOLS_OK: &[(&str, &str)] = &[
    ("de", "Du darfst Home-Assistant-Werkzeuge nutzen, wenn der Nutzer das Haus oder aktuelle Fakten braucht."),
    ("en", "You may use Home Assistant tools when the user needs the house or current facts."),
];

const NEWS: &[(&str, &str)] = &[
    (
        "de",
        "Der Nutzer will aktuelle Nachrichten. Fasse die folgenden Schlagzeilen in drei bis fünf kurzen Sätzen zusammen. Erfinde keine Meldungen. Wenn du Websuche oder Nachrichten-Werkzeuge hast, nutze sie. Frage am Ende, ob der Nutzer zu einer Meldung mehr erfahren möchte.",
    ),
    (
        "en",
        "The user wants current news. Summarize the following headlines in three to five short sentences. Do not invent stories. If you have web search or news tools, use them. End by asking whether they want more detail on any story.",
    ),
    ("ja", "利用者はニュースを求めています。次の見出しを三〜五文で要約してください。作り話はしないでください。最後に、詳しく聞きたいか尋ねてください。"),
    ("zh-CN", "用户想听新闻。用三到五句概括下面的标题。不要编造。最后问要不要了解其中一条。"),
    ("ko", "사용자가 뉴스를 원합니다. 아래 헤드라인을 세다섯 문장으로 요약하세요. 지어내지 마세요. 끝에 더 들을지 물으세요."),
    ("hi", "उपयोगकर्ता समाचार चाहता है। नीचे की सुर्खियाँ तीन से पाँच वाक्यों में सार दो। कुछ गढ़ो मत। अंत में पूछो कि और सुनना है या नहीं।"),
    ("ar", "يريد المستخدم أخباراً. لخص العناوين التالية في ثلاث إلى خمس جمل. لا تختلق أخباراً. اسأل في النهاية إن أراد المزيد."),
    ("th", "ผู้ใช้ต้องการข่าว สรุปหัวข้อต่อไปนี้สามถึงห้าประโยค อย่าแต่งข่าว ท้ายถามว่าอยากฟังเพิ่มไหม"),
];

const NEWS_FOLLOW: &[(&str, &str)] = &[
    ("de", "Bleib beim Nachrichtenthema. Antworte knapp. Steuere keine Geräte."),
    ("en", "Stay on the news topic. Keep it short. Do not control devices."),
    ("ja", "ニュースの話題から外れないでください。短く。機器は操作しないでください。"),
    ("zh-CN", "继续谈新闻。简短回答。不要控制设备。"),
    ("ko", "뉴스 주제를 유지하세요. 짧게. 기기를 제어하지 마세요."),
    ("hi", "समाचार पर रहो। संक्षेप में। उपकरण मत चलाओ।"),
    ("ar", "ابقَ عند موضوع الأخبار. أجب باختصار. لا تتحكم في الأجهزة."),
    ("th", "อยู่ที่ข่าว ตอบสั้น อย่าควบคุมอุปกรณ์"),
];

const CALENDAR: &[(&str, &str, &str)] = &[
    ("de", "Der Nutzer fragt nach seinem Kalender. Formuliere die folgenden Termine natürlich und knapp. Erfinde keine Termine. Wenn die Liste leer ist, sag das klar. Kein Wetter und keine Vorhersage.", "Termine"),
    ("en", "The user is asking about their calendar. Say the following events naturally and briefly. Do not invent events. If the list is empty, say so clearly. Do not report weather or a forecast.", "Events"),
    ("ja", "利用者はカレンダーを尋ねています。次の予定を自然に短く言ってください。予定を作らないでください。空ならそう言ってください。天気は言わないでください。", "予定"),
];

const CALENDAR_NO_WEATHER: &[(&str, &str)] = &[
    ("de", "Kein Wetter, keine Vorhersage. Keine Home-Assistant-Werkzeuge."),
    ("en", "Do not report weather or a forecast. Do not call Home Assistant tools."),
];

pub fn pick<'a>(table: &'a [(&'a str, &'a str)], pack: &str, fallback: &'a str) -> &'a str {
    table.iter().find(|(code, _)| *code == pack).map(|(_, text)| *text).unwrap_or(fallback)
}

fn answer_in_pack(pack: &str) -> String {
    format!(
        "Reply in conversation only. Do not control devices and do not call Home Assistant tools. Answer in the user's language (Assist pack code: {pack})."
    )
}

pub fn with_personality(base: &str, voice: &str) -> String {
    let voice = voice.trim();
    let base = base.trim();
    if !voice.is_empty() && !base.is_empty() {
        format!("{voice}\n\n{base}")
    } else if !voice.is_empty() {
        voice.to_string()
    } else {
        base.to_string()
    }
}

pub fn chat_only_prompt(pack: &str, extra: Option<&str>, allow_tools: bool) -> String {
    let only = if allow_tools {
        pick(TOOLS_OK, pack, TOOLS_OK[1].1).to_string()
    } else {
        CHAT_ONLY.iter().find(|(code, _)| *code == pack).map(|(_, text)| (*text).to_string()).unwrap_or_else(|| answer_in_pack(pack))
    };
    wrap(pack, extra, &only)
}

pub fn yarn_prompt(pack: &str, extra: Option<&str>, text: &str) -> String {
    chat_only_prompt(pack, Some(&join_extra(extra, yarn_body(pack, text))), false)
}

pub fn news_prompt(pack: &str, headlines: &[String], extra: Option<&str>) -> String {
    let fallback = format!(
        "The user wants current news. Summarize the following headlines in three to five short sentences. Do not invent stories. If you have web search or news tools, use them. Answer in the user's language (Assist pack code: {pack})."
    );
    let mut body = pick(NEWS, pack, &fallback).to_string();
    if !headlines.is_empty() {
        let lines = headlines.iter().map(|item| format!("- {item}")).collect::<Vec<_>>().join("\n");
        let label = if pack == "de" || pack.starts_with("de-") { "Schlagzeilen" } else { "Headlines" };
        body = format!("{body}\n\n{label}:\n{lines}");
    }
    chat_only_prompt(pack, Some(&join_extra(extra, &body)), false)
}

pub fn news_followup_prompt(pack: &str, extra: Option<&str>) -> String {
    let fallback =
        format!("Stay on the news topic. Keep it short. Do not control devices. Answer in the user's language (Assist pack code: {pack}).");
    chat_only_prompt(pack, Some(&join_extra(extra, pick(NEWS_FOLLOW, pack, &fallback))), false)
}

pub fn calendar_prompt(pack: &str, facts: &str, extra: Option<&str>) -> String {
    let (body, label) = calendar_copy(pack);
    let mut body = body.to_string();
    let facts = facts.trim();
    if !facts.is_empty() {
        body = format!("{body}\n\n{label}:\n{facts}");
    }
    let lock = language_lock(pack);
    let extra = extra.unwrap_or("").trim();
    let stay = pick(CALENDAR_NO_WEATHER, pack, CALENDAR_NO_WEATHER[1].1);
    let core = if extra.is_empty() { body } else { format!("{extra}\n{body}") };
    format!("{lock}\n{core}\n{stay}\n{lock}")
}

pub fn calendar_readback(pack: &str, facts: &str) -> String {
    let events = if facts.trim().is_empty() {
        if pack == "de" || pack.starts_with("de-") { "Keine Termine." } else { "No events." }.to_string()
    } else {
        facts.trim().to_string()
    };
    if pack == "de" || pack.starts_with("de-") {
        format!("Lies nur diese Kalendertermine vor:\n{events}")
    } else {
        format!("Read back only these calendar events:\n{events}")
    }
}

pub fn history_prompt(pack: &str, turns: &[(String, String)]) -> String {
    if turns.is_empty() {
        return String::new();
    }
    let german = pack == "de" || pack.starts_with("de-");
    let header = if german {
        "Bisher im Gespräch (behalte Thema und Auftrag, auch bei kurzen Antworten wie egal):"
    } else {
        "Conversation so far (keep the topic and task, including short replies like whatever):"
    };
    let who = if german { "Nutzer" } else { "User" };
    let mut lines = vec![header.to_string()];
    for (user, assistant) in turns {
        if !user.is_empty() {
            lines.push(format!("{who}: {user}"));
        }
        if !assistant.is_empty() {
            lines.push(format!("Klar: {assistant}"));
        }
    }
    lines.join("\n")
}

pub fn keeps_calendar_reply(facts: &str, llm: &str) -> bool {
    let speech = llm.trim();
    if speech.is_empty() || speech.contains('?') {
        return false;
    }
    !super::refine_accept::weather_claim(speech) || super::refine_accept::weather_claim(facts)
}

pub fn system_for(
    pack: &str,
    kind: AssistKind,
    text: &str,
    extra: Option<&str>,
    allow_tools: bool,
    retrieval: Option<&serde_json::Value>,
    facts: &[String],
) -> String {
    match kind {
        AssistKind::Yarn => yarn_prompt(pack, extra, text),
        AssistKind::Rag => rag_prompt(pack, retrieval, extra),
        AssistKind::News => news_prompt(pack, facts, extra),
        AssistKind::NewsFollow => news_followup_prompt(pack, extra),
        AssistKind::Calendar => calendar_prompt(pack, &facts.join("\n"), extra),
        AssistKind::Chat | AssistKind::Auto => {
            if yarn_request(text) {
                yarn_prompt(pack, extra, text)
            } else {
                chat_only_prompt(pack, extra, allow_tools)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssistKind {
    Auto,
    Yarn,
    Chat,
    Rag,
    Calendar,
    News,
    NewsFollow,
}

impl AssistKind {
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        match raw.trim() {
            "" | "auto" => Ok(Self::Auto),
            "yarn" => Ok(Self::Yarn),
            "chat" => Ok(Self::Chat),
            "rag" => Ok(Self::Rag),
            "calendar" => Ok(Self::Calendar),
            "news" => Ok(Self::News),
            "news_follow" => Ok(Self::NewsFollow),
            _ => Err("kind"),
        }
    }

    pub fn resolve(self, text: &str, nlu_rag: bool) -> Self {
        match self {
            Self::Auto if yarn_request(text) => Self::Yarn,
            Self::Auto if nlu_rag => Self::Rag,
            Self::Auto => Self::Chat,
            other => other,
        }
    }
}

fn wrap(pack: &str, extra: Option<&str>, body: &str) -> String {
    let lock = language_lock(pack);
    let extra = extra.unwrap_or("").trim();
    let body = if extra.is_empty() { body.to_string() } else { format!("{extra}\n{body}") };
    format!("{lock}\n{body}\n{lock}")
}

fn join_extra(extra: Option<&str>, body: &str) -> String {
    let extra = extra.unwrap_or("").trim();
    if extra.is_empty() {
        body.to_string()
    } else {
        format!("{extra}\n{body}")
    }
}

fn calendar_copy(pack: &str) -> (&'static str, &'static str) {
    CALENDAR.iter().find(|(code, _, _)| *code == pack).map(|(_, body, label)| (*body, *label)).unwrap_or((CALENDAR[1].1, CALENDAR[1].2))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_only_and_yarn_prompts() {
        let prompt = chat_only_prompt("de", Some("Sei kurz."), false);
        assert!(prompt.contains("Sei kurz."));
        assert!(prompt.contains("keine Home-Assistant-Werkzeuge"));
        let sw = chat_only_prompt("sw", None, false);
        assert!(!sw.contains("Steuere keine Geräte"));
        assert!(sw.contains("Assist pack code: sw"));
        let ja = chat_only_prompt("ja", None, false);
        assert!(ja.contains("会話"));
        assert!(!ja.contains("Steuere keine Geräte"));
        let allowed = chat_only_prompt("en", None, true);
        assert!(allowed.contains("You may use Home Assistant tools"));
        assert!(!allowed.contains("Do not control devices"));
        let story = yarn_prompt("de", None, "Erzähle eine Geschichte");
        assert!(story.contains("Antwort = die Geschichte selbst"));
        assert!(!story.contains("Erzähl jetzt einen Witz"));
        assert_eq!(with_personality("Nur reden.", "Stimme: Butler."), "Stimme: Butler.\n\nNur reden.");
        assert_eq!(AssistKind::parse("auto").unwrap().resolve("erzähl einen Witz", false), AssistKind::Yarn);
        assert_eq!(AssistKind::parse("auto").unwrap().resolve("licht an", true), AssistKind::Rag);
    }

    #[test]
    fn calendar_keeps_facts_and_rejects_weather() {
        let prompt = calendar_prompt("en", "dentist tomorrow at 3", None);
        assert!(prompt.contains("dentist tomorrow at 3"));
        assert!(prompt.contains("Do not invent events"));
        assert!(prompt.contains("Do not report weather"));
        let ja = calendar_prompt("ja", "meeting", None);
        assert!(ja.contains("予定"));
        assert!(!ja.contains("Termine"));
        assert!(keeps_calendar_reply("dentist is tomorrow.", "Dentist is tomorrow."));
        assert!(!keeps_calendar_reply("Nothing tomorrow.", "Tomorrow will be sunny with 18°C."));
        let asked = calendar_readback("en", "dentist is tomorrow at 3.");
        assert!(asked.starts_with("Read back only these calendar events"));
        assert!(history_prompt("de", &[("erzähl eine Geschichte".into(), "Kurz oder lang?".into())]).starts_with("Bisher im Gespräch"));
    }
}
