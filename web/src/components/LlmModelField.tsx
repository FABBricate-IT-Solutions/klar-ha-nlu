import { useEffect, useMemo, useState } from "react";
import type { Messages } from "../i18n";
import { listLlmModels } from "../llmModels";
import type { LlmProviderId } from "../llmProviders";
import { SearchSelect, withCurrent } from "./SearchSelect";

export function LlmModelField({
  t,
  baseUrl,
  apiKey,
  model,
  onModel,
  inputId = "llm-model",
  provider,
}: {
  t: Messages;
  baseUrl: string;
  apiKey: string;
  model: string;
  onModel: (value: string) => void;
  inputId?: string;
  provider?: LlmProviderId;
}) {
  const [models, setModels] = useState<string[]>([]);
  const [status, setStatus] = useState<"idle" | "loading" | "empty" | "fail">("idle");

  useEffect(() => {
    if (!baseUrl.trim()) {
      setModels([]);
      setStatus("idle");
      return;
    }
    let cancelled = false;
    setStatus("loading");
    const timer = window.setTimeout(() => {
      listLlmModels({
        baseUrl: baseUrl.trim(),
        apiKey,
        provider: provider ?? "custom",
      })
        .then((ids) => {
          if (cancelled) return;
          setModels(ids);
          setStatus(ids.length ? "idle" : "empty");
        })
        .catch(() => {
          if (cancelled) return;
          setModels([]);
          setStatus("fail");
        });
    }, 400);
    return () => {
      cancelled = true;
      window.clearTimeout(timer);
    };
  }, [apiKey, baseUrl, provider]);

  const options = useMemo(
    () => withCurrent(models.map((id) => ({ value: id, label: id })), model),
    [model, models],
  );
  const hint = (() => {
    switch (status) {
      case "loading":
        return t.llmModelsLoading;
      case "fail":
        return t.llmModelsFail;
      case "empty":
        return t.llmModelsEmpty;
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
        allowEmpty={false}
        allowCustom
        placeholder="gpt-4o-mini"
      />
      {hint ? <p className="caption">{hint}</p> : null}
    </>
  );
}
