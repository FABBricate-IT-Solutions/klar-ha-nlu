import { useCallback, useState } from "react";
import { guessProvider, needsKey, providerById, providerForUrl, sameBase, type LlmProviderId } from "./llmProviders";
import type { LlmPublic } from "./types";

export type LlmForm = ReturnType<typeof useLlmForm>;

/** The LLM endpoint form behind Settings and the setup wizard. */
export function useLlmForm() {
  const [saved, setSaved] = useState<LlmPublic>({ configured: false });
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [thinking, setThinking] = useState(false);
  const [provider, setProvider] = useState<LlmProviderId>("custom");

  /** Fills the form from what the engine has stored. The key never comes back from the engine. */
  const load = useCallback((next: LlmPublic) => {
    setSaved(next);
    setBaseUrl(next.base_url || "");
    setModel(next.model || "");
    setApiKey("");
    setThinking(Boolean(next.enable_thinking));
    setProvider(providerForUrl(next.base_url || "", next));
  }, []);

  /** A changed URL decides the provider again; a provider picked by hand holds until then. */
  const setUrl = (value: string) => {
    setBaseUrl(value);
    setProvider(providerForUrl(value, saved));
  };

  const pickProvider = (id: LlmProviderId) => {
    const preset = providerById(id);
    if (!preset || id === provider) return;
    setProvider(id);
    if (id === "custom" || guessProvider(baseUrl) === id) return;
    setBaseUrl(preset.url);
    if (preset.model) setModel(preset.model);
  };

  // The engine keeps the stored key for the stored address only. Once the URL names another
  // address, "leave empty to keep the stored key" no longer holds.
  const moved = Boolean(saved.configured && saved.base_url && baseUrl.trim() && !sameBase(baseUrl, saved.base_url));
  const keyRequired = moved && needsKey(provider);

  return {
    saved,
    baseUrl,
    model,
    apiKey,
    thinking,
    provider,
    /** The URL is not the stored address any more, so the stored key will not be used for it. */
    moved,
    /** The provider at the new address needs a key. */
    keyRequired,
    /** The form must not be saved yet: it would store a cloud endpoint without its key. */
    keyMissing: keyRequired && !apiKey.trim(),
    load,
    setUrl,
    setModel,
    setKey: setApiKey,
    setThinking,
    pickProvider,
    /** The body for `POST /api/v2/llm/endpoint`. An empty key field sends no key. */
    payload: () => ({
      base_url: baseUrl,
      model,
      ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
      configured: true,
      enable_thinking: thinking,
      provider,
    }),
  };
}
