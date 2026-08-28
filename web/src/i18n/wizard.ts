/** First-run wizard copy. Chrome packs can later spread these into de.ts / en.ts. */

export type WizardPhrase = { say: string; expect: string };

export type WizardMessages = {
  title: string;
  skip: string;
  back: string;
  next: string;
  done: string;
  stepOf: string;
  close: string;
  detected: string;
  recommended: string;

  whatTitle: string;
  whatLead: string;
  whatLocal: string;
  whatConsole: string;
  whatNoLlm: string;

  pathTitle: string;
  pathLead: string;
  pathShared: string;
  pathAddonTitle: string;
  pathAddonBody: string;
  pathDockerTitle: string;
  pathDockerBody: string;
  pathBinaryTitle: string;
  pathBinaryBody: string;
  pathSampleTitle: string;
  pathSampleBody: string;

  modeTitle: string;
  modeLead: string;
  modeFullTitle: string;
  modeFullBody: string;
  modeContextTitle: string;
  modeContextBody: string;
  modeNluTitle: string;
  modeNluBody: string;

  missTitle: string;
  missLead: string;
  missEngineTitle: string;
  missEngineBody: string;
  missSliceTitle: string;
  missSliceBody: string;
  missLlmTitle: string;
  missLlmBody: string;
  missWarn: string;

  toolsTitle: string;
  toolsLead: string;
  toolsLab: string;
  toolsMapping: string;
  toolsPhrases: string;
  toolsRoutines: string;
  toolsPolicies: string;

  phrasesTitle: string;
  phrasesLead: string;
  phrasesOther: string;
  phrasesMapping: string;
  phrasesReopen: string;
  phraseSay: string;
  phraseExpect: string;
  phrases: WizardPhrase[];
};

export const wizardDe: WizardMessages = {
  title: "Setup",
  skip: "Überspringen",
  back: "Zurück",
  next: "Weiter",
  done: "Fertig",
  stepOf: "Schritt {n} von {total}",
  close: "Schließen",
  detected: "Erkannt",
  recommended: "Empfohlen",

  whatTitle: "Was Klar ist",
  whatLead:
    "Lokale Assist-NLU. Home Assistant bleibt die Gerätedatenbank. Das hier ist keine Haushaltszentrale und keine LLM-Conversation-Engine.",
  whatLocal: "Klar zerlegt den Satz. Home Assistant besitzt Räume und Geräte.",
  whatConsole: "Lovelace „Klar“ ist der letzte Assist-Zug. Diese Oberfläche (Klar NLU) ist Zuordnung und Labor.",
  whatNoLlm: "Den LLM-Agenten nicht als Conversation-Engine in der Pipeline setzen.",

  pathTitle: "Installationsweg",
  pathLead: "Ein Engine-Host. Integration für Assist, App oder Docker für diese UI.",
  pathShared: "App und mitgelieferte Engine nicht gleichzeitig. Beides macht das Parsen nicht genauer.",
  pathAddonTitle: "Home Assistant App",
  pathAddonBody:
    "Du bist über Ingress in der App. Integration synchronisiert das Haus. URL in der Integration: http://klar-nlu:10520.",
  pathDockerTitle: "App oder Docker",
  pathDockerBody:
    "Der Graph kommt von Home Assistant. In der Integration „Klar-NLU-App oder Docker“ wählen. Mapping und Labor laufen hier.",
  pathBinaryTitle: "Mitgelieferte Engine",
  pathBinaryBody:
    "HACS-only oder Binary auf Loopback (127.0.0.1:10520). Assist funktioniert. Zuordnung und Labor erreicht ein Telefon nicht.",
  pathSampleTitle: "Beispielhaus",
  pathSampleBody:
    "Noch das eingebaute default_home, kein HA-Snapshot. Assist-Pipeline speichern — dann erscheint euer Haus.",

  modeTitle: "Modus",
  modeLead: "Drei getrennte Schalter. Keiner davon ist ein LLM in Assist.",
  modeFullTitle: "Geräte auflösen",
  modeFullBody: "full bindet eine entity_id. Das ist der normale Haushalt.",
  modeContextTitle: "Nur Räume",
  modeContextBody: "context_only stoppt am Raum. Kein Gerät, kein Schalten über den Resolver.",
  modeNluTitle: "Nur NLU",
  modeNluBody: "Standard für unbehandelte Sprache: matched slice, keine Assist-Werkzeuge, kein Hauskontext.",

  missTitle: "Wenn Klar danebenliegt",
  missLead: "Drei Schichten. Nur die erste sitzt in dieser Engine.",
  missEngineTitle: "Klar-Engine",
  missEngineBody: "Ausführen, nachfragen, ablehnen. Labor zeigt die Pipeline.",
  missSliceTitle: "Hauskontext bei einem Miss",
  missSliceBody:
    "Optional. Schon gematchte Namen, Räume, letzter Zug. Keine Embeddings, kein Dokumentindex, keine Assist-Tools. Standard aus.",
  missLlmTitle: "HA-LLM",
  missLlmBody: "Fallback und Refine bleiben in der Integration. Assist-Tools am LLM-Agenten aus.",
  missWarn: "LLM niemals in den Slot „Conversation-Engine“ der Pipeline.",

  toolsTitle: "Werkzeuge",
  toolsLead: "Nach dem Setup bleibt ihr in dieser Konsole, nicht in den API-Docs.",
  toolsLab: "Labor — Satz nochmal, Pipeline und verworfene Kandidaten.",
  toolsMapping: "Zuordnung — Aliase und Raumvorschläge. Overlay über HA-Namen.",
  toolsPhrases: "Sätze — ein Satz auf einen bekannten Intent. Nicht für jedes Licht.",
  toolsRoutines: "Routinen — ein gesprochener Name startet ein Skript.",
  toolsPolicies: "Policies — erste zutreffende Regel gewinnt. Gerät, Raum oder Etage wählen.",

  phrasesTitle: "Fünf Sätze",
  phrasesLead: "Nach der Pipeline in Assist sagen. Unter HA OS auch im Labor.",
  phrasesOther: "Englisch läuft in derselben Pipeline.",
  phrasesMapping: "{count} Geräte ohne sicheren Raum — als Nächstes Haus → Zuordnung.",
  phrasesReopen: "Dieses Setup liegt unter Einstellungen, nicht als siebter Tab.",
  phraseSay: "Sagen",
  phraseExpect: "Erwartung",
  phrases: [
    { say: "Licht im Wohnzimmer an", expect: "Wohnzimmerlicht an" },
    { say: "Garagentor auf 40 %", expect: "Cover-Position 40 %" },
    { say: "Mach das Licht aus und die Heizung auf 21", expect: "Zwei Schritte: Licht aus, Klima 21" },
    { say: "Wohnzimmer Fernseher pausieren", expect: "Media-Pause auf dem Player" },
    { say: "Spiel Queen", expect: "Music Assistant sucht und spielt" },
  ],
};

export const wizardEn: WizardMessages = {
  title: "Setup",
  skip: "Skip",
  back: "Back",
  next: "Next",
  done: "Done",
  stepOf: "Step {n} of {total}",
  close: "Close",
  detected: "Detected",
  recommended: "Recommended",

  whatTitle: "What Klar is",
  whatLead:
    "Local Assist NLU. Home Assistant stays the device database. This is not a household dashboard and not an LLM conversation engine.",
  whatLocal: "Klar parses the sentence. Home Assistant owns rooms and devices.",
  whatConsole: "Lovelace “Klar” is the last Assist turn. This surface (Klar NLU) is Mapping and Lab.",
  whatNoLlm: "Do not set an LLM as the pipeline conversation engine.",

  pathTitle: "Install path",
  pathLead: "One engine host. Integration for Assist. App or Docker for this UI.",
  pathShared: "Do not run the App and the bundled engine together. Installing both does not make parsing more accurate.",
  pathAddonTitle: "Home Assistant App",
  pathAddonBody:
    "You reached the App through ingress. The integration syncs the house. Integration URL: http://klar-nlu:10520.",
  pathDockerTitle: "App or Docker",
  pathDockerBody:
    "The graph was pushed from Home Assistant. In the integration pick “Use the Klar NLU App or Docker”. Mapping and Lab run here.",
  pathBinaryTitle: "Bundled engine",
  pathBinaryBody:
    "HACS-only or a binary on loopback (127.0.0.1:10520). Assist works. A phone cannot reach Mapping or Lab.",
  pathSampleTitle: "Sample house",
  pathSampleBody:
    "Still the built-in default_home, not an HA snapshot. Save the Assist pipeline — then your house appears.",

  modeTitle: "Mode",
  modeLead: "Three separate switches. None of them puts an LLM in Assist.",
  modeFullTitle: "Resolve devices",
  modeFullBody: "full binds an entity_id. That is the usual household.",
  modeContextTitle: "Rooms only",
  modeContextBody: "context_only stops at the room. No device, no power through the resolver.",
  modeNluTitle: "NLU only",
  modeNluBody: "Default for unhandled speech: the matched slice, no Assist tools, no house context.",

  missTitle: "On a miss",
  missLead: "Three layers. Only the first lives in this engine.",
  missEngineTitle: "Klar engine",
  missEngineBody: "Execute, clarify, or reject. Lab shows the pipeline.",
  missSliceTitle: "House context on miss",
  missSliceBody:
    "Optional. Already-matched names, rooms, last turn. No embeddings, no document index, no Assist tools. Default off.",
  missLlmTitle: "HA LLM",
  missLlmBody: "Fallback and refine stay in the integration. Keep Assist tools off on the LLM agent.",
  missWarn: "Never put the LLM in the pipeline Conversation engine slot.",

  toolsTitle: "Tools",
  toolsLead: "After setup you stay in this console, not the API docs.",
  toolsLab: "Lab — replay a sentence, pipeline, discarded candidates.",
  toolsMapping: "Mapping — aliases and room suggestions. Overlay on HA names.",
  toolsPhrases: "Phrases — one sentence to a known intent. Not one row per lamp.",
  toolsRoutines: "Routines — a spoken name starts a script.",
  toolsPolicies: "Policies — first matching rule wins. Pick device, room, or floor.",

  phrasesTitle: "Five phrases",
  phrasesLead: "Say them in Assist after the pipeline is saved. On HA OS, Lab works too.",
  phrasesOther: "German works in the same pipeline.",
  phrasesMapping: "{count} devices without a sure room — next is House → Mapping.",
  phrasesReopen: "Re-open this setup from Settings. It is not a seventh tab.",
  phraseSay: "Say",
  phraseExpect: "Expect",
  phrases: [
    { say: "Turn on the living room light", expect: "Living-room lights on" },
    { say: "Set the garage door to 40%", expect: "Cover position 40%" },
    { say: "Turn the lights off and set heat to 21", expect: "Two steps: lights off, climate 21" },
    { say: "Pause the living room TV", expect: "Media pause on that player" },
    { say: "Play Queen", expect: "Music Assistant search-and-play" },
  ],
};

export function wizardMessages(locale?: string, extra?: Partial<WizardMessages>): WizardMessages {
  const base = (locale || "").toLowerCase().startsWith("de") ? wizardDe : wizardEn;
  if (!extra) return base;
  return { ...base, ...extra, phrases: extra.phrases ?? base.phrases };
}

export function fillWizard(template: string, slots: Record<string, string>): string {
  return Object.entries(slots).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, value), template);
}
