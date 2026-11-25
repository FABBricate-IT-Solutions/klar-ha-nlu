import { toast } from "sonner";
import type { Messages } from "./i18n";
import { ApiError, isApiError } from "./lib/apiError";

function errorText(err: unknown): string {
  return err instanceof Error ? err.message : String(err || "");
}

function statusOf(err: unknown): number {
  return isApiError(err) ? err.status : 0;
}

export type SaveFailKind = "settings" | "generic";

export function saveFailMessage(t: Messages, err: unknown, kind: SaveFailKind = "generic"): string {
  const status = statusOf(err);
  const text = errorText(err);
  if (status === 401 || text.startsWith("401")) return t.saveUnauthorized;
  // Settings 422 is the engine refusing an experimental language pick (#415).
  if (kind === "settings" && (status === 422 || text.startsWith("422"))) return t.saveExperimentalRefused;
  // Prefer the engine's `{ error }` text when it sent one (U-18).
  if (err instanceof ApiError && err.engineError && !/^\d{3}\b/.test(err.engineError)) return err.engineError;
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
  const status = statusOf(err);
  const text = errorText(err);
  const title = saveFailMessage(t, err, kind);
  if (status === 401 || status === 422 || text.startsWith("401") || text.startsWith("422") || !text) {
    toast.error(title);
    return;
  }
  const description =
    err instanceof ApiError && err.path
      ? text === title
        ? err.path
        : `${err.path}: ${text}`
      : text === title
        ? undefined
        : text;
  if (description) {
    toast.error(title, { description });
    return;
  }
  toast.error(title);
}

/** Surface a non-save failure (dashboard refresh, apply, …) with the engine text when present. */
export function toastActionFailed(err: unknown, fallback = "Request failed"): void {
  if (err instanceof ApiError) {
    toast.error(err.engineError || fallback, err.path ? { description: err.path } : undefined);
    return;
  }
  toast.error(errorText(err) || fallback);
}
