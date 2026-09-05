"""Chat-only fallback. Prompt text is not a tool lock."""

from __future__ import annotations

import re

# homeassistant.components.conversation.ConversationEntityFeature.CONTROL
_CONTROL = 1

_CHAT_ONLY = {
    "de": (
        "Du antwortest nur im Gespräch. Steuere keine Geräte "
        "und rufe keine Home-Assistant-Werkzeuge auf."
    ),
    "en": (
        "Reply in conversation only. Do not control devices "
        "and do not call Home Assistant tools."
    ),
    "ja": "会話だけで答えてください。機器は操作せず、Home Assistantのツールも呼ばないでください。",
    "zh-CN": "只对话回答。不要控制设备，也不要调用 Home Assistant 工具。",
    "zh-TW": "只對話回答。不要控制裝置，也不要呼叫 Home Assistant 工具。",
    "zh-HK": "只對話答。唔好控制裝置，亦唔好叫 Home Assistant 工具。",
    "ko": "대화로만 답하세요. 기기를 제어하지 말고 Home Assistant 도구도 호출하지 마세요.",
    "hi": "केवल बातचीत में जवाब दो। उपकरण मत चलाओ और Home Assistant औज़ार मत बुलाओ।",
    "ar": "أجب في المحادثة فقط. لا تتحكم في الأجهزة ولا تستدع أدوات Home Assistant.",
    "th": "ตอบเฉพาะบทสนทนา อย่าควบคุมอุปกรณ์ และอย่าเรียกเครื่องมือ Home Assistant",
}

_ANSWER_IN_PACK = (
    "Reply in conversation only. Do not control devices "
    "and do not call Home Assistant tools. "
    "Answer in the user's language (Assist pack code: {pack})."
)

_TOOLS_OK = {
    "de": (
        "Du darfst Home-Assistant-Werkzeuge nutzen, "
        "wenn der Nutzer das Haus oder aktuelle Fakten braucht."
    ),
    "en": (
        "You may use Home Assistant tools when the user needs "
        "the house or current facts."
    ),
}


def agent_has_home_control(features: object) -> bool:
    try:
        flag = int(features)  # type: ignore[arg-type]
    except (TypeError, ValueError):
        return True
    return bool(flag & _CONTROL)


_NEWS = {
    "de": (
        "Der Nutzer will aktuelle Nachrichten. "
        "Fasse die folgenden Schlagzeilen in drei bis fünf kurzen Sätzen zusammen. "
        "Erfinde keine Meldungen. "
        "Wenn du Websuche oder Nachrichten-Werkzeuge hast, nutze sie. "
        "Frage am Ende, ob der Nutzer zu einer Meldung mehr erfahren möchte."
    ),
    "en": (
        "The user wants current news. "
        "Summarize the following headlines in three to five short sentences. "
        "Do not invent stories. "
        "If you have web search or news tools, use them. "
        "End by asking whether they want more detail on any story."
    ),
    "ja": "利用者はニュースを求めています。次の見出しを三〜五文で要約してください。作り話はしないでください。最後に、詳しく聞きたいか尋ねてください。",
    "zh-CN": "用户想听新闻。用三到五句概括下面的标题。不要编造。最后问要不要了解其中一条。",
    "ko": "사용자가 뉴스를 원합니다. 아래 헤드라인을 세다섯 문장으로 요약하세요. 지어내지 마세요. 끝에 더 들을지 물으세요.",
    "hi": "उपयोगकर्ता समाचार चाहता है। नीचे की सुर्खियाँ तीन से पाँच वाक्यों में सार दो। कुछ गढ़ो मत। अंत में पूछो कि और सुनना है या नहीं।",
    "ar": "يريد المستخدم أخباراً. لخص العناوين التالية في ثلاث إلى خمس جمل. لا تختلق أخباراً. اسأل في النهاية إن أراد المزيد.",
    "th": "ผู้ใช้ต้องการข่าว สรุปหัวข้อต่อไปนี้สามถึงห้าประโยค อย่าแต่งข่าว ท้ายถามว่าอยากฟังเพิ่มไหม",
}

_NEWS_FOLLOW = {
    "de": "Bleib beim Nachrichtenthema. Antworte knapp. Steuere keine Geräte.",
    "en": "Stay on the news topic. Keep it short. Do not control devices.",
    "ja": "ニュースの話題から外れないでください。短く。機器は操作しないでください。",
    "zh-CN": "继续谈新闻。简短回答。不要控制设备。",
    "ko": "뉴스 주제를 유지하세요. 짧게. 기기를 제어하지 마세요.",
    "hi": "समाचार पर रहो। संक्षेप में। उपकरण मत चलाओ।",
    "ar": "ابقَ عند موضوع الأخبار. أجب باختصار. لا تتحكم في الأجهزة.",
    "th": "อยู่ที่ข่าว ตอบสั้น อย่าควบคุมอุปกรณ์",
}

_CALENDAR = {
    "de": (
        "Der Nutzer fragt nach seinem Kalender. "
        "Formuliere die folgenden Termine natürlich und knapp. "
        "Erfinde keine Termine. Wenn die Liste leer ist, sag das klar. "
        "Kein Wetter und keine Vorhersage."
    ),
    "en": (
        "The user is asking about their calendar. "
        "Say the following events naturally and briefly. "
        "Do not invent events. If the list is empty, say so clearly. "
        "Do not report weather or a forecast."
    ),
}

_CALENDAR_NO_WEATHER = {
    "de": "Kein Wetter, keine Vorhersage. Keine Home-Assistant-Werkzeuge.",
    "en": "Do not report weather or a forecast. Do not call Home Assistant tools.",
}

_WEATHER_WORDS = (
    "weather",
    "forecast",
    "degrees",
    "celsius",
    "fahrenheit",
    "humidity",
    "precipitation",
    "sunny",
    "cloudy",
    "rain",
    "rainy",
    "raining",
    "wetter",
    "vorhersage",
    "regen",
    "sonnig",
    "regnerisch",
    "bewölkt",
    "bewolkt",
)
_WEATHER_STEMS = ("°c", "°f", "luftfeucht")
_WEATHER_WORD = re.compile(r"\b(?:" + "|".join(_WEATHER_WORDS) + r")\b")


def weather_claim(text: str) -> bool:
    """True when speech reports a forecast. 'training' must not count as 'rain'."""
    fold = (text or "").casefold()
    if any(stem in fold for stem in _WEATHER_STEMS):
        return True
    return bool(_WEATHER_WORD.search(fold))


def _calendar_copy(pack: str) -> tuple[str, str]:
    try:
        from .calendar_say import llm_copy

        return llm_copy(pack)
    except ImportError:
        try:
            from calendar_say import llm_copy

            return llm_copy(pack)
        except ImportError:
            body = _CALENDAR.get(pack) or _CALENDAR["en"]
            return body, {"de": "Termine", "en": "Events"}.get(pack, "Events")


def _language_lock(pack: str) -> str:
    try:
        from .lang_select import language_lock

        return language_lock(pack)
    except ImportError:
        if pack == "de" or pack.startswith("de-"):
            return "Antworte nur auf Deutsch. Übersetze nicht in eine andere Sprache."
        return (
            f"Answer only in the Klar NLU language ({pack}). "
            "Do not translate into German or any other language."
        )


_STORY = {
    "de": (
        "Der Nutzer will eine Geschichte. Erzähl jetzt eine kurze Geschichte (ein Beat). "
        "Antwort = die Geschichte selbst. Keine Frage. Nicht um Erlaubnis fragen. Kein Witz. "
        "Verboten: Soll ich, Darf ich, Womit soll, Kurz oder lang, Welche Art. "
        "Ein oder zwei Sätze, dann Schluss. Keine Geräte, keine entity_id."
    ),
    "en": (
        "The user wants a story. Tell a short story now (one beat). "
        "Your reply is the story itself. No question. Do not ask permission. Do not tell a joke. "
        "Forbidden: shall I, should I, do you want, what kind. "
        "One or two sentences, then stop. No devices, no entity ids."
    ),
}

_JOKE = {
    "de": (
        "Der Nutzer will einen Witz. Erzähl jetzt einen Witz. Nicht fragen. Keine Geschichte. "
        "Ein oder zwei Sätze, dann Schluss. Keine Geräte, keine entity_id."
    ),
    "en": (
        "The user wants a joke. Tell a joke now. Do not ask. Do not tell a story. "
        "One or two sentences, then stop. No devices, no entity ids."
    ),
}

_YARN = {
    "de": _STORY["de"] + " " + _JOKE["de"],
    "en": _STORY["en"] + " " + _JOKE["en"],
}

_STORY_WORDS = (
    "geschichte",
    "story",
    "stories",
    "fairytale",
    "märchen",
    "maerchen",
)
_JOKE_WORDS = ("witz", "joke", "jokes")
_PERMISSION = (
    "soll ich",
    "darf ich",
    "womit soll",
    "kurz oder lang",
    "welche art",
    "wollen sie",
    "möchtest",
    "moechtest",
    "möchten sie",
    "moechten sie",
    "kann dir gerne",
    "kann ich dir",
    "gerne eine geschichte",
    "shall i",
    "should i",
    "do you want",
    "would you like",
    "what kind",
    "want me to",
    "i can tell",
    "i'd be happy",
)
_CANNED_STORY = {
    "de": "Es war einmal ein Fuchs, der nachts über den stillen Hof lief und den Mond begrüßte, bevor er wieder im Wald verschwand.",
    "en": "Once there was a fox who crossed a quiet yard at night, nodded to the moon, and slipped back into the woods.",
}
_CANNED_JOKE = {
    "de": "Warum tragen Geister keine Hüte? Weil sie durch sind.",
    "en": "Why don't ghosts wear hats? They go right through them.",
}


def joke_request(text: str) -> bool:
    blob = (text or "").casefold()
    return any(word in blob for word in _JOKE_WORDS)


def story_request(text: str) -> bool:
    if joke_request(text):
        return False
    blob = (text or "").casefold()
    return any(word in blob for word in _STORY_WORDS)


def yarn_request(text: str) -> bool:
    return story_request(text) or joke_request(text)


def yarn_asks_permission(speech: str) -> bool:
    blob = (speech or "").casefold()
    return any(phrase in blob for phrase in _PERMISSION)


def yarn_canned(pack: str, text: str) -> str:
    if joke_request(text):
        return _CANNED_JOKE.get(pack) or _CANNED_JOKE["en"]
    return _CANNED_STORY.get(pack) or _CANNED_STORY["en"]


def yarn_nudge(pack: str, prompt: str) -> str:
    extra = (
        "Erzähl jetzt. Keine Frage. Beginne mit der Geschichte oder dem Witz."
        if pack == "de" or pack.startswith("de-")
        else "Tell it now. No question. Start with the story or joke."
    )
    return f"{prompt}\n{extra}"


def yarn_prompt(pack: str, extra: str | None, text: str | None = None) -> str:
    if text and joke_request(text):
        body = _JOKE.get(pack) or _JOKE["en"]
    elif text and story_request(text):
        body = _STORY.get(pack) or _STORY["en"]
    else:
        body = _YARN.get(pack) or _YARN["en"]
    return chat_only_prompt(pack, _join_extra(extra, body))


def chat_only_prompt(pack: str, extra: str | None, allow_tools: bool = False) -> str:
    only = (
        (_TOOLS_OK.get(pack) or _TOOLS_OK["en"])
        if allow_tools
        else (_CHAT_ONLY.get(pack) or _ANSWER_IN_PACK.format(pack=pack))
    )
    lock = _language_lock(pack)
    extra = (extra or "").strip()
    body = f"{extra}\n{only}" if extra else only
    return f"{lock}\n{body}\n{lock}"


def with_personality(base: str | None, voice: str | None) -> str:
    voice = (voice or "").strip()
    base = (base or "").strip()
    if voice and base:
        return f"{voice}\n\n{base}"
    return voice or base


def news_prompt(pack: str, headlines: list[str], extra: str | None) -> str:
    body = _NEWS.get(pack) or (
        "The user wants current news. "
        "Summarize the following headlines in three to five short sentences. "
        "Do not invent stories. "
        "If you have web search or news tools, use them. "
        f"Answer in the user's language (Assist pack code: {pack})."
    )
    if headlines:
        lines = "\n".join(f"- {item}" for item in headlines)
        label = {"de": "Schlagzeilen", "en": "Headlines"}.get(pack, "Headlines")
        body = f"{body}\n\n{label}:\n{lines}"
    return chat_only_prompt(pack, _join_extra(extra, body))


def news_followup_prompt(pack: str, extra: str | None) -> str:
    stay = _NEWS_FOLLOW.get(pack) or (
        f"Stay on the news topic. Keep it short. Do not control devices. "
        f"Answer in the user's language (Assist pack code: {pack})."
    )
    return chat_only_prompt(pack, _join_extra(extra, stay))


def calendar_query_only(intents: list | None) -> bool:
    if not intents:
        return False
    return all(item.get("name") == "KlarGetCalendarEvents" for item in intents)


def calendar_prompt(pack: str, facts: str, extra: str | None = None) -> str:
    body, label = _calendar_copy(pack)
    facts = (facts or "").strip()
    if facts:
        body = f"{body}\n\n{label}:\n{facts}"
    lock = _language_lock(pack)
    extra = (extra or "").strip()
    stay = _CALENDAR_NO_WEATHER.get(pack) or _CALENDAR_NO_WEATHER["en"]
    core = f"{extra}\n{body}" if extra else body
    return f"{lock}\n{core}\n{stay}\n{lock}"


def calendar_readback(pack: str, facts: str) -> str:
    events = (facts or "").strip() or (
        "Keine Termine." if pack == "de" or pack.startswith("de-") else "No events."
    )
    if pack == "de" or pack.startswith("de-"):
        return f"Lies nur diese Kalendertermine vor:\n{events}"
    return f"Read back only these calendar events:\n{events}"


def keeps_calendar_reply(facts: str, llm: str) -> bool:
    speech = (llm or "").strip()
    if not speech or "?" in speech:
        return False
    return not weather_claim(speech) or weather_claim(facts)


def _join_extra(extra: str | None, body: str) -> str:
    extra = (extra or "").strip()
    return f"{extra}\n{body}" if extra else body


def llm_conversation_id(session_key: str) -> str:
    key = (session_key or "klar-followup").strip() or "klar-followup"
    return f"klar-llm-{key}"[:128]


def append_llm_turn(
    turns: list[tuple[str, str]] | None, user: str, assistant: str, keep: int = 8
) -> list[tuple[str, str]]:
    out = list(turns or [])
    user, assistant = user.strip(), assistant.strip()
    if user or assistant:
        out.append((user, assistant))
    return out[-keep:]


def history_prompt(pack: str, turns: list[tuple[str, str]] | None) -> str:
    if not turns:
        return ""
    german = pack == "de" or pack.startswith("de-")
    header = (
        "Bisher im Gespräch (behalte Thema und Auftrag, auch bei kurzen Antworten wie egal):"
        if german
        else "Conversation so far (keep the topic and task, including short replies like whatever):"
    )
    who = "Nutzer" if german else "User"
    lines = [header]
    for user, assistant in turns:
        if user:
            lines.append(f"{who}: {user}")
        if assistant:
            lines.append(f"Klar: {assistant}")
    return "\n".join(lines)


def can_use_fallback_agent(
    controls_home: bool, chat: bool = False, allow_tools: bool = False
) -> bool:
    del chat
    return allow_tools or not controls_home
