import type { PolicyEffect, SpeechBankEntry } from "./types";

const VOICES: Record<string, Record<string, Record<PolicyEffect, string[]>>> = {
  default: {
    de: {
      confirm: ["Soll ich {name} wirklich schalten?", "Einmal bestätigen: {name} in {area}.", "{name} — darf ich das tun?"],
      block: ["Das lasse ich.", "{name} bleibt unangetastet.", "Nicht in {area}."],
      allow: ["{name} ist erledigt.", "In {area}: {name}.", "Passt."],
      prefer_entity: ["Ich nehme {name}.", "{name} ist die bevorzugte Wahl.", "Richtung {name}."],
      prefer_area: ["Ich bleibe in {area}.", "{area} zuerst.", "Raum {area}."],
      reply: ["Verstanden.", "Alles klar.", "Notiert."],
      script: ["Ich starte das Skript.", "Skript läuft.", "Erledigt."],
      template: ["Einen Moment.", "Ich schaue nach.", "Kurz prüfen."],
      llm: ["Ich denke nach.", "Einen Augenblick.", "Ich formuliere das."],
    },
    en: {
      confirm: ["Should I really change {name}?", "Confirm {name} in {area}.", "{name} — may I?"],
      block: ["I will leave that.", "{name} stays as it is.", "Not in {area}."],
      allow: ["{name} is done.", "In {area}: {name}.", "Set."],
      prefer_entity: ["I will use {name}.", "{name} is preferred.", "Toward {name}."],
      prefer_area: ["Staying in {area}.", "{area} first.", "Room {area}."],
      reply: ["Understood.", "All right.", "Noted."],
      script: ["Starting the script.", "Script is running.", "Done."],
      template: ["One moment.", "Let me check.", "Looking that up."],
      llm: ["Thinking.", "One moment.", "Let me phrase that."],
    },
  },
  butler: {
    de: {
      confirm: ["Darf ich {name} für Sie schalten?", "Eine Bestätigung zu {name} in {area}, bitte.", "Soll ich {name} in die Wege leiten?"],
      block: ["Das werde ich nicht tun.", "{name} bleibt, wie es ist.", "In {area} greife ich nicht ein."],
      allow: ["{name} ist besorgt.", "In {area} ist {name} erledigt.", "Sehr wohl."],
      prefer_entity: ["Ich wähle {name}.", "{name}, wie vorgesehen.", "Bevorzugt: {name}."],
      prefer_area: ["Ich bleibe im {area}.", "{area}, selbstverständlich.", "Raum {area}."],
      reply: ["Sehr wohl.", "Wie Sie wünschen.", "Notiert."],
      script: ["Ich starte das Skript.", "Das Skript läuft.", "Erledigt."],
      template: ["Einen Moment, bitte.", "Ich sehe nach.", "Sofort."],
      llm: ["Ich formuliere das.", "Einen Augenblick.", "Sofort."],
    },
    en: {
      confirm: ["May I change {name} for you?", "A confirmation for {name} in {area}.", "Shall I proceed with {name}?"],
      block: ["I will not do that.", "{name} remains as it is.", "I will not act in {area}."],
      allow: ["{name} is taken care of.", "In {area}, {name} is done.", "Very well."],
      prefer_entity: ["I shall use {name}.", "{name}, as preferred.", "Preferred: {name}."],
      prefer_area: ["I remain in {area}.", "{area}, of course.", "Room {area}."],
      reply: ["Very well.", "As you wish.", "Noted."],
      script: ["I shall start the script.", "The script is running.", "Done."],
      template: ["One moment, please.", "I will check.", "At once."],
      llm: ["I shall phrase that.", "One moment.", "At once."],
    },
  },
};

const FALLBACK = VOICES.default;

export function bakeVariants(ruleId: string, effect: PolicyEffect, personality: string, languages: string[]): SpeechBankEntry {
  const voice = VOICES[personality] || FALLBACK;
  const variants = languages.flatMap((language) => {
    const pack = voice[language] || FALLBACK[language] || FALLBACK.de;
    return (pack[effect] || FALLBACK.de[effect]).map((text) => ({ language, personality, text }));
  });
  return { rule_id: ruleId, variants: variants.slice(0, 5) };
}
