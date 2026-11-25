/** Typed failure from the operator API (status + optional engine `{ error, path }`). */
export class ApiError extends Error {
  readonly status: number;
  readonly path: string;
  readonly engineError: string;

  constructor(status: number, statusText: string, body?: { error?: unknown; path?: unknown } | null) {
    const engineError =
      typeof body?.error === "string" && body.error.trim()
        ? body.error.trim()
        : `${status} ${statusText}`.trim();
    super(engineError);
    this.name = "ApiError";
    this.status = status;
    this.path = typeof body?.path === "string" ? body.path : "";
    this.engineError = engineError;
  }
}

export function isApiError(err: unknown): err is ApiError {
  return err instanceof ApiError;
}

/** Read a JSON error body when the engine sent one; otherwise null. */
export async function readErrorBody(res: Response): Promise<{ error?: unknown; path?: unknown } | null> {
  const text = await res.text().catch(() => "");
  if (!text) return null;
  try {
    const row = JSON.parse(text) as unknown;
    if (!row || typeof row !== "object") return null;
    return row as { error?: unknown; path?: unknown };
  } catch {
    return { error: text };
  }
}
