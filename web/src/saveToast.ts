import { toast } from "sonner";
import type { Messages } from "./i18n";

function errorText(err: unknown): string {
  return err instanceof Error ? err.message : String(err || "");
}

export function saveFailMessage(t: Messages, err: unknown): string {
  return errorText(err).startsWith("401") ? t.saveUnauthorized : t.saveFail;
}

export function toastSaved(t: Messages, detail?: string): void {
  if (detail) {
    toast.success(t.saveOk, { description: detail });
    return;
  }
  toast.success(t.saveOk);
}

export function toastSaveFailed(t: Messages, err: unknown): void {
  const text = errorText(err);
  const title = saveFailMessage(t, err);
  if (text.startsWith("401") || !text) {
    toast.error(title);
    return;
  }
  toast.error(title, { description: text });
}
