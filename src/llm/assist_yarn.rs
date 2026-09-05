//! Yarn / joke / story classification and canned replies. No model call.

const STORY_WORDS: &[&str] = &["geschichte", "story", "stories", "fairytale", "märchen", "maerchen"];
const JOKE_WORDS: &[&str] = &["witz", "joke", "jokes"];
const PERMISSION: &[&str] = &[
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
];

const STORY_DE: &str = "Der Nutzer will eine Geschichte. Erzähl jetzt eine kurze Geschichte (ein Beat). \
Antwort = die Geschichte selbst. Keine Frage. Nicht um Erlaubnis fragen. Kein Witz. \
Verboten: Soll ich, Darf ich, Womit soll, Kurz oder lang, Welche Art. \
Ein oder zwei Sätze, dann Schluss. Keine Geräte, keine entity_id.";
const STORY_EN: &str = "The user wants a story. Tell a short story now (one beat). \
Your reply is the story itself. No question. Do not ask permission. Do not tell a joke. \
Forbidden: shall I, should I, do you want, what kind. \
One or two sentences, then stop. No devices, no entity ids.";
const JOKE_DE: &str = "Der Nutzer will einen Witz. Erzähl jetzt einen Witz. Nicht fragen. Keine Geschichte. \
Ein oder zwei Sätze, dann Schluss. Keine Geräte, keine entity_id.";
const JOKE_EN: &str = "The user wants a joke. Tell a joke now. Do not ask. Do not tell a story. \
One or two sentences, then stop. No devices, no entity ids.";
const CANNED_STORY_DE: &str =
    "Es war einmal ein Fuchs, der nachts über den stillen Hof lief und den Mond begrüßte, bevor er wieder im Wald verschwand.";
const CANNED_STORY_EN: &str =
    "Once there was a fox who crossed a quiet yard at night, nodded to the moon, and slipped back into the woods.";
const CANNED_JOKE_DE: &str = "Warum tragen Geister keine Hüte? Weil sie durch sind.";
const CANNED_JOKE_EN: &str = "Why don't ghosts wear hats? They go right through them.";

pub fn joke_request(text: &str) -> bool {
    let blob = text.to_lowercase();
    JOKE_WORDS.iter().any(|word| blob.contains(word))
}

pub fn story_request(text: &str) -> bool {
    if joke_request(text) {
        return false;
    }
    let blob = text.to_lowercase();
    STORY_WORDS.iter().any(|word| blob.contains(word))
}

pub fn yarn_request(text: &str) -> bool {
    story_request(text) || joke_request(text)
}

pub fn yarn_asks_permission(speech: &str) -> bool {
    let blob = speech.to_lowercase();
    PERMISSION.iter().any(|phrase| blob.contains(phrase))
}

pub fn yarn_canned(pack: &str, text: &str) -> String {
    if joke_request(text) {
        if is_de(pack) { CANNED_JOKE_DE } else { CANNED_JOKE_EN }.to_string()
    } else if is_de(pack) {
        CANNED_STORY_DE.to_string()
    } else {
        CANNED_STORY_EN.to_string()
    }
}

pub fn yarn_nudge(pack: &str, prompt: &str) -> String {
    let extra = if is_de(pack) {
        "Erzähl jetzt. Keine Frage. Beginne mit der Geschichte oder dem Witz."
    } else {
        "Tell it now. No question. Start with the story or joke."
    };
    format!("{prompt}\n{extra}")
}

pub fn yarn_body(pack: &str, text: &str) -> &'static str {
    if joke_request(text) {
        if is_de(pack) {
            JOKE_DE
        } else {
            JOKE_EN
        }
    } else if story_request(text) {
        if is_de(pack) {
            STORY_DE
        } else {
            STORY_EN
        }
    } else if is_de(pack) {
        // Combined yarn when the request is ambiguous.
        STORY_DE
    } else {
        STORY_EN
    }
}

fn is_de(pack: &str) -> bool {
    pack == "de" || pack.starts_with("de-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_joke_story_and_canned() {
        assert!(story_request("Erzähle eine Geschichte"));
        assert!(!joke_request("Erzähle eine Geschichte"));
        assert!(yarn_request("Tell me a joke"));
        assert!(!yarn_request("Licht an"));
        assert!(yarn_asks_permission("Soll ich dir eine kurze Geschichte erzählen?"));
        assert!(yarn_asks_permission("Ich kann dir gerne eine Geschichte erzählen."));
        assert!(!yarn_asks_permission("Es war einmal ein Fuchs im Wald."));
        assert!(yarn_canned("de", "Erzähle eine Geschichte").contains("Fuchs"));
        assert!(yarn_canned("de", "Erzähle einen Witz").contains("Geister"));
        assert!(!yarn_canned("de", "Erzähle eine Geschichte").contains("Soll ich"));
        assert!(yarn_body("de", "Erzähle eine Geschichte").contains("Antwort = die Geschichte selbst"));
        assert!(!yarn_body("de", "Erzähle eine Geschichte").contains("Erzähl jetzt einen Witz"));
        assert!(yarn_body("de", "Erzähle einen Witz").contains("Witz"));
    }
}
