import { useMemo, useState } from "react";
import type { Messages } from "../i18n";
import { listLlmModels } from "../llmModels";
import type { LlmProviderId } from "../llmProviders";
import { SearchSelect, withCurrent } from "./SearchSelect";
import { Button } from "@/components/ui/button";

type ListingStatus = "idle" | "loading" | "ready" | "empty" | "fail";
type Listing = { for: string; models: string[]; status: Exclude<ListingStatus, "idle"> };

/**
 * The model picker. The list is asked from the engine only when the operator opens it or uses
 * Load models / Retry, and only once per URL, key and provider. Typing in the URL or key field
 * asks nothing: the model list request is what could carry a key to the address in the URL field.
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
  const status: ListingStatus = current?.status ?? "idle";

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

  const { hint, action } = (() => {
    switch (status) {
      case "loading":
        return { hint: t.llmModelsLoading, action: null as string | null };
      case "fail":
        return { hint: t.llmModelsFail, action: t.llmRetry };
      case "empty":
        return { hint: t.llmModelsEmpty, action: null };
      case "ready":
        return { hint: "", action: null };
      case "idle":
        return { hint: "", action: t.llmLoadModels };
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
      {action ? (
        <Button
          type="button"
          variant="link"
          size="xs"
          className="h-auto px-0"
          disabled={!baseUrl.trim()}
          onClick={load}
        >
          {action}
        </Button>
      ) : null}
    </>
  );
}
