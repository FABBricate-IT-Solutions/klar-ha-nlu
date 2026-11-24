import { useMemo, useState } from "react";
import type { Messages } from "../i18n";
import { listLlmModels } from "../llmModels";
import type { LlmProviderId } from "../llmProviders";
import { SearchSelect, withCurrent } from "./SearchSelect";

type Listing = { for: string; models: string[]; status: "loading" | "ready" | "empty" | "fail" };

/**
 * The model picker. The list is asked from the engine only when the operator opens it, and only
 * once per URL, key and provider. Typing in the URL or key field asks nothing: the model list
 * request is what could carry a key to the address in the URL field.
 */
export function LlmModelField({
  t,
  baseUrl,
  apiKey,
  model,
  onModel,
  inputId = "llm-model",
  provider = "custom",
}: {
  t: Messages;
  baseUrl: string;
  apiKey: string;
  model: string;
  onModel: (value: string) => void;
  inputId?: string;
  provider?: LlmProviderId;
}) {
  const [listing, setListing] = useState<Listing | null>(null);
  const wanted = `${provider}\n${baseUrl.trim()}\n${apiKey}`;
  // A list loaded for another URL, key or provider is not this endpoint's list.
  const current = listing?.for === wanted ? listing : null;
  const status = current?.status ?? "idle";

  const load = () => {
    // Opening the list again repeats only a failed attempt.
    if (!baseUrl.trim() || (status !== "idle" && status !== "fail")) return;
    const settle = (next: Listing) => setListing((prev) => (prev?.for === wanted ? next : prev));
    setListing({ for: wanted, models: [], status: "loading" });
    listLlmModels({ baseUrl, apiKey, provider })
      .then((models) => settle({ for: wanted, models, status: models.length ? "ready" : "empty" }))
      .catch(() => settle({ for: wanted, models: [], status: "fail" }));
  };

  const options = useMemo(
    () => withCurrent((current?.models ?? []).map((id) => ({ value: id, label: id })), model),
    [current, model],
  );
  const hint = (() => {
    switch (status) {
      case "loading":
        return t.llmModelsLoading;
      case "fail":
        return t.llmModelsFail;
      case "empty":
        return t.llmModelsEmpty;
      case "ready":
      case "idle":
        return "";
      default: {
        const _never: never = status;
        return _never;
      }
    }
  })();

  return (
    <>
      <SearchSelect
        id={inputId}
        value={model}
        options={options}
        onChange={onModel}
        onOpen={load}
        allowEmpty={false}
        allowCustom
        placeholder="gpt-4o-mini"
      />
      {hint ? <p className="caption" role="status">{hint}</p> : null}
    </>
  );
}
