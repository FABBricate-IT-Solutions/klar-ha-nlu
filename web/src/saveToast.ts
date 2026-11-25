import { toast } from "sonner";
import type { Messages } from "./i18n";

function errorText(err: unknown): string {
  return err instanceof Error ? err.message : String(err || "");
}

export type SaveFailKind = "settings" | "generic";

export function saveFailMessage(t: Messages, err: unknown, kind: SaveFailKind = "generic"): string {
  const text = errorText(err);
  if (text.startsWith("401")) return t.saveUnauthorized;
  // Settings 422 is the engine refusing an experimental language pick (#415).
  if (kind === "settings" && text.startsWith("422")) return t.saveExperimentalRefused;
  return t.saveFail;
}

export function toastSaved(t: Messages, detail?: string): void {
  if (detail) {
    toast.success(t.saveOk, { description: detail });
    return;
  }
  toast.success(t.saveOk);
}

export function toastSaveFailed(t: Messages, err: unknown, kind: SaveFailKind = "generic"): void {
  const text = errorText(err);
  const title = saveFailMessage(t, err, kind);
  if (text.startsWith("401") || text.startsWith("422") || !text) {
    toast.error(title);
    return;
  }
  toast.error(title, { description: text });
}
