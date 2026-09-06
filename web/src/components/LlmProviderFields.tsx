import { useEffect, useState } from "react";
import type { Messages } from "../i18n";
import {
  exactProvider,
  guessProvider,
  LLM_PROVIDERS,
  providerById,
  resolveProvider,
  writeStoredProvider,
  type LlmProviderId,
} from "../llmProviders";
import { SearchSelect } from "./SearchSelect";
import { LlmModelField } from "./LlmModelField";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";

function providerLabel(t: Messages, id: LlmProviderId): string {
  switch (id) {
    case "openai":
      return t.llmPresetOpenAi;
    case "anthropic":
      return t.llmPresetAnthropic;
    case "google":
      return t.llmPresetGoogle;
    case "lemonade":
      return t.llmPresetLemonade;
    case "llamacpp":
      return t.llmPresetLlamaCpp;
    case "custom":
      return t.llmPresetCustom;
    default: {
      const _never: never = id;
      return _never;
    }
  }
}

export function LlmProviderFields({
  t,
  baseUrl,
  model,
  apiKey,
  thinking,
  onUrl,
  onModel,
  onKey,
  onThinking,
  modelInputId,
  idPrefix = "",
}: {
  t: Messages;
  baseUrl: string;
  model: string;
  apiKey: string;
  thinking: boolean;
  onUrl: (value: string) => void;
  onModel: (value: string) => void;
  onKey: (value: string) => void;
  onThinking: (value: boolean) => void;
  modelInputId?: string;
  idPrefix?: string;
}) {
  const [provider, setProvider] = useState<LlmProviderId>(() => resolveProvider(baseUrl));
  useEffect(() => {
    const exact = exactProvider(baseUrl);
    if (exact !== "custom") {
      setProvider(exact);
      writeStoredProvider(exact);
      return;
    }
    const next = resolveProvider(baseUrl);
    setProvider(next);
    writeStoredProvider(next);
  }, [baseUrl]);
  const urlId = `${idPrefix}llm-base-url`;
  const keyId = `${idPrefix}llm-key`;
  const modelId = modelInputId || `${idPrefix}llm-model`;
  const preset = providerById(provider);
  return (
    <FieldGroup>
      <Field>
        <FieldLabel>{t.llmProvider}</FieldLabel>
        <SearchSelect
          value={provider}
          options={LLM_PROVIDERS.map((row) => ({ value: row.id, label: providerLabel(t, row.id) }))}
          onChange={(value) => {
            const next = providerById(value);
            if (!next || next.id === provider) return;
            setProvider(next.id);
            writeStoredProvider(next.id);
            if (next.id === "custom") return;
            if (guessProvider(baseUrl) === next.id) return;
            onUrl(next.url);
            if (next.model) onModel(next.model);
          }}
          allowEmpty={false}
          placeholder={t.llmProvider}
        />
      </Field>
      <Field>
        <FieldLabel htmlFor={urlId}>{t.llmBaseUrl}</FieldLabel>
        <Input
          id={urlId}
          value={baseUrl}
          onChange={(ev) => onUrl(ev.target.value)}
          placeholder={preset?.url || "https://api.openai.com/v1"}
        />
      </Field>
      <Field>
        <FieldLabel htmlFor={modelId}>{t.llmModel}</FieldLabel>
        <LlmModelField
          t={t}
          baseUrl={baseUrl}
          apiKey={apiKey}
          model={model}
          onModel={onModel}
          inputId={modelId}
          provider={provider}
        />
      </Field>
      <Field>
        <FieldLabel htmlFor={keyId}>{t.llmApiKey}</FieldLabel>
        <Input
          id={keyId}
          type="password"
          value={apiKey}
          onChange={(ev) => onKey(ev.target.value)}
          placeholder={t.llmApiKeyHint}
          autoComplete="off"
        />
      </Field>
      <Field orientation="horizontal">
        <FieldContent>
          <FieldLabel>{t.llmThinking}</FieldLabel>
          <FieldDescription>{t.llmThinkingHint}</FieldDescription>
        </FieldContent>
        <Switch checked={thinking} onCheckedChange={(checked) => onThinking(Boolean(checked))} />
      </Field>
    </FieldGroup>
  );
}
