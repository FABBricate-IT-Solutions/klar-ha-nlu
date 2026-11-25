import type { Messages } from "../i18n";
import { LLM_PROVIDERS, providerById, type LlmProviderId } from "../llmProviders";
import type { LlmForm } from "../useLlmForm";
import { SearchSelect } from "./SearchSelect";
import { LlmModelField } from "./LlmModelField";
import { SettingsToggle } from "./SettingsToggle";
import { Field, FieldError, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";

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
  form,
  modelInputId,
  idPrefix = "",
}: {
  t: Messages;
  form: LlmForm;
  modelInputId?: string;
  idPrefix?: string;
}) {
  const { baseUrl, model, apiKey, thinking, provider } = form;
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
            if (next) form.pickProvider(next.id);
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
          onChange={(ev) => form.setUrl(ev.target.value)}
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
          onModel={form.setModel}
          inputId={modelId}
          provider={provider}
        />
      </Field>
      <Field data-invalid={form.keyMissing || undefined}>
        <FieldLabel htmlFor={keyId}>{t.llmApiKey}</FieldLabel>
        {/* The stored key stays with the stored address. For another address the field stops
            promising it, and a cloud provider's key becomes required. */}
        <Input
          id={keyId}
          type="password"
          value={apiKey}
          onChange={(ev) => form.setKey(ev.target.value)}
          placeholder={form.moved ? t.llmApiKey : t.llmApiKeyHint}
          required={form.keyRequired}
          aria-invalid={form.keyMissing || undefined}
          aria-describedby={form.keyMissing ? `${keyId}-required` : undefined}
          autoComplete="off"
        />
        {form.keyMissing ? (
          <FieldError id={`${keyId}-required`}>{t.llmApiKeyRequired}</FieldError>
        ) : null}
      </Field>
      <SettingsToggle
        id="klar-llm-thinking"
        label={t.llmThinking}
        description={t.llmThinkingHint}
        checked={thinking}
        onCheckedChange={(checked) => form.setThinking(checked)}
      />
    </FieldGroup>
  );
}
