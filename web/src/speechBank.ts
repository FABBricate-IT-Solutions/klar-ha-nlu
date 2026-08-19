import type { PolicyEffect, SpeechBankEntry } from "./types";

type Lines = Record<PolicyEffect, string[]>;

const DE_DEFAULT: Lines = {
  confirm: ["Soll ich {name} wirklich schalten?", "Einmal bestätigen: {name} in {area}.", "{name} — darf ich das tun?"],
  block: ["Das lasse ich.", "{name} bleibt unangetastet.", "Nicht in {area}."],
  allow: ["{name} ist erledigt.", "In {area}: {name}.", "Passt."],
  prefer_entity: ["Ich nehme {name}.", "{name} ist die bevorzugte Wahl.", "Richtung {name}."],
  prefer_area: ["Ich bleibe in {area}.", "{area} zuerst.", "Raum {area}."],
  reply: ["Verstanden.", "Alles klar.", "In Ordnung."],
  script: ["Ich starte das Skript.", "Skript läuft.", "Erledigt."],
  template: ["Einen Moment.", "Ich schaue nach.", "Kurz prüfen."],
  llm: ["Ich denke nach.", "Einen Augenblick.", "Ich formuliere das."],
};

const EN_DEFAULT: Lines = {
  confirm: ["Should I really change {name}?", "Confirm {name} in {area}.", "{name} — may I?"],
  block: ["I will leave that.", "{name} stays as it is.", "Not in {area}."],
  allow: ["{name} is done.", "In {area}: {name}.", "Set."],
  prefer_entity: ["I will use {name}.", "{name} is preferred.", "Toward {name}."],
  prefer_area: ["Staying in {area}.", "{area} first.", "Room {area}."],
  reply: ["Understood.", "All right.", "Got it."],
  script: ["Starting the script.", "Script is running.", "Done."],
  template: ["One moment.", "Let me check.", "Looking that up."],
  llm: ["Thinking.", "One moment.", "Let me phrase that."],
};

function mix(base: Lines, reply: string[], allow: string[], confirm?: string[]): Lines {
  return { ...base, reply, allow, confirm: confirm || base.confirm };
}

const VOICES: Record<string, Record<string, Lines>> = {
  default: { de: DE_DEFAULT, en: EN_DEFAULT },
  butler: {
    de: mix(DE_DEFAULT, ["Sehr wohl.", "Gern.", "Wie Sie wünschen."], ["{name} ist erledigt.", "In {area} ist {name} fertig.", "Gern erledigt."], ["Darf ich {name} für Sie schalten?", "Eine Bestätigung zu {name} in {area}, bitte.", "Soll ich {name} anfassen?"]),
    en: mix(EN_DEFAULT, ["Very well.", "Gladly.", "As you wish."], ["{name} is done.", "In {area}, {name} is done.", "Gladly done."], ["May I change {name} for you?", "A confirmation for {name} in {area}.", "Shall I proceed with {name}?"]),
  },
  locker: {
    de: mix(DE_DEFAULT, ["Geht klar.", "Passt.", "Alles klar."], ["{name} ist durch.", "In {area}: {name}.", "Passt."]),
    en: mix(EN_DEFAULT, ["Got it.", "Done.", "All good."], ["{name} is done.", "In {area}: {name}.", "All set."]),
  },
  fuersorglich: {
    de: mix(DE_DEFAULT, ["Alles gut.", "Ich mach das.", "Ruhig."], ["{name} ist erledigt, alles gut.", "In {area} ist {name} fertig.", "Du musst nichts tun."]),
    en: mix(EN_DEFAULT, ["All good.", "Doing that now.", "Easy."], ["{name} is done, all good.", "In {area}, {name} is done.", "You can relax."]),
  },
  party: {
    de: mix(DE_DEFAULT, ["Läuft.", "Schön.", "Genau so."], ["{name} ist durch.", "In {area}: {name}.", "Läuft."]),
    en: mix(EN_DEFAULT, ["Let's go.", "Nice.", "Love it."], ["{name} is done.", "In {area}: {name}.", "Let's go."]),
  },
  grantig: {
    de: mix(DE_DEFAULT, ["Schon gut.", "Wenn's sein muss.", "Na gut."], ["{name} ist gemacht.", "In {area}: {name}.", "Hab ich gemacht."]),
    en: mix(EN_DEFAULT, ["Fine.", "If I must.", "Right."], ["{name} is done.", "In {area}: {name}.", "I did it."]),
  },
  sarkastisch: {
    de: mix(DE_DEFAULT, ["Na klar.", "Was für eine Überraschung.", "Natürlich."], ["{name} ist erledigt.", "In {area}: {name}.", "Überraschung."]),
    en: mix(EN_DEFAULT, ["Of course.", "What a surprise.", "Naturally."], ["{name} is done.", "In {area}: {name}.", "Shocking."]),
  },
  pirat: {
    de: mix(DE_DEFAULT, ["Aye.", "Käpt'n.", "Klar."], ["{name} ist gesetzt.", "In {area}: {name}.", "Aye."]),
    en: mix(EN_DEFAULT, ["Aye.", "Captain.", "Set."], ["{name} is set.", "In {area}: {name}.", "Aye."]),
  },
  hippie: {
    de: mix(DE_DEFAULT, ["Alles easy.", "Ganz ruhig.", "Easy."], ["{name} ist erledigt.", "In {area}: {name}.", "Alles easy."]),
    en: mix(EN_DEFAULT, ["All good.", "Easy.", "Peace."], ["{name} is done.", "In {area}: {name}.", "Easy."]),
  },
  gollum: {
    de: mix(DE_DEFAULT, ["Ja.", "Ja, mein Schatz.", "Mhm."], ["{name} ist erledigt.", "In {area}: {name}.", "Ja."]),
    en: mix(EN_DEFAULT, ["Yes.", "Yes, my precious.", "Yes."], ["{name} is done.", "In {area}: {name}.", "Yes."]),
  },
};

const FALLBACK = VOICES.default;

export function bakeVariants(ruleId: string, effect: PolicyEffect, personality: string, languages: string[]): SpeechBankEntry {
  const voice = VOICES[personality] || FALLBACK;
  const variants = languages.flatMap((language) => {
    const pack = voice[language] || FALLBACK[language] || FALLBACK.en;
    return (pack[effect] || FALLBACK.en[effect]).map((text) => ({ language, personality, text }));
  });
  return { rule_id: ruleId, variants: variants.slice(0, 5) };
}
