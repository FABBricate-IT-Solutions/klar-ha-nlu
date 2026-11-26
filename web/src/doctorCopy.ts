/** Doctor chrome stays out of the 65-locale catalog — only en/de here to keep packs lean. */

export type DoctorCopy = {
  nav: string;
  title: string;
  hint: string;
  build: string;
  home: string;
  overlay: string;
  llm: string;
  wyoming: string;
  ops: string;
  unavailable: string;
  entities: string;
  age: string;
  ageNever: string;
};

const EN: DoctorCopy = {
  nav: "Doctor",
  title: "Doctor",
  hint: "Live health from the engine: build identity, home graph, overlay, LLM, Wyoming.",
  build: "Build",
  home: "Home graph",
  overlay: "Overlay",
  llm: "LLM",
  wyoming: "Wyoming",
  ops: "Ops",
  unavailable: "Health not reachable.",
  entities: "{n} entities",
  age: "{n}s since last sync",
  ageNever: "No live sync yet",
};

const DE: DoctorCopy = {
  nav: "Doctor",
  title: "Doctor",
  hint: "Live-Health der Engine: Build-Identität, Home-Graph, Overlay, LLM, Wyoming.",
  build: "Build",
  home: "Home-Graph",
  overlay: "Overlay",
  llm: "LLM",
  wyoming: "Wyoming",
  ops: "Ops",
  unavailable: "Health nicht erreichbar.",
  entities: "{n} Entitäten",
  age: "{n}s seit letztem Sync",
  ageNever: "Noch kein Live-Sync",
};

export function doctorCopy(locale: string): DoctorCopy {
  return locale.toLowerCase().startsWith("de") ? DE : EN;
}

export function fillDoctor(template: string, vars: Record<string, string>): string {
  return template.replace(/\{(\w+)\}/g, (_, key: string) => vars[key] ?? `{${key}}`);
}
