import { useCallback, useEffect, useRef, useState } from "react";
import { api, type LangOverlay } from "./api";

/** Live engine overlay. Failed or stale tabs do not keep the last phrase list. */
export function useLangOverlay() {
  const [overlay, setOverlay] = useState<LangOverlay | null>(null);
  const [offline, setOffline] = useState(false);
  const [status, setStatus] = useState("");
  /** While true, focus/visibility must not refetch (JSON editor has unsaved text). */
  const holdRef = useRef(false);

  const refresh = useCallback(async () => {
    try {
      const next = await api.langOverlay();
      setOverlay(next);
      setOffline(false);
      setStatus("");
      return next;
    } catch (err) {
      setOverlay(null);
      setOffline(true);
      setStatus(String(err));
      return null;
    }
  }, []);

  const hold = useCallback((next: boolean) => {
    holdRef.current = next;
  }, []);

  useEffect(() => {
    void refresh();
    const onShow = () => {
      if (holdRef.current) return;
      if (document.visibilityState === "visible") void refresh();
    };
    window.addEventListener("focus", onShow);
    document.addEventListener("visibilitychange", onShow);
    return () => {
      window.removeEventListener("focus", onShow);
      document.removeEventListener("visibilitychange", onShow);
    };
  }, [refresh]);

  return { overlay, offline, status, setStatus, refresh, replace: setOverlay, hold };
}
