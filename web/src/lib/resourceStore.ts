/** Compare two JSON-serialisable values for a dirty check. */
export function sameJson(left: unknown, right: unknown): boolean {
  return JSON.stringify(left) === JSON.stringify(right);
}

/**
 * Keep a draft in sync with a saved value until the operator edits it.
 * Returns the next draft: the saved value when clean, otherwise the previous draft.
 */
export function syncDraftWhenClean<T>(saved: T, draft: T, dirty: boolean): T {
  return dirty ? draft : saved;
}
