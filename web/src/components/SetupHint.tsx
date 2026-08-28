import type { Messages } from "../i18n";

function isDe(t: Messages): boolean {
  return t.replay === "Nochmal";
}

export function sentencesEmpty(t: Messages): string {
  return isDe(t)
    ? "Ein Satz auf einen bekannten Intent. Nicht für jedes Licht."
    : "One sentence onto a known intent. Not one phrase per light.";
}

export function policiesEmpty(t: Messages): string {
  return isDe(t)
    ? "Erste zutreffende Regel gewinnt. Gerät/Raum/Etage wählen, nicht tippen."
    : "First matching rule wins. Pick a device, room, or floor — do not type ids.";
}

export function setupAgainWhere(t: Messages): string {
  return isDe(t)
    ? "Setup erneut liegt unter Einstellungen."
    : "Replay setup is in Settings.";
}

export function SetupHint({ t }: { t: Messages }) {
  return <p className="caption">{setupAgainWhere(t)}</p>;
}
