import { LlmModelField } from "../LlmModelField";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import { field, control } from "./styles";

export function LlmStep({
  copy,
  chrome,
  url,
  model,
  apiKey,
  thinking,
  onUrl,
  onModel,
  onKey,
  onThinking,
}: {
  copy: WizardMessages;
  chrome: Messages;
  url: string;
  model: string;
  apiKey: string;
  thinking: boolean;
  onUrl: (value: string) => void;
  onModel: (value: string) => void;
  onKey: (value: string) => void;
  onThinking: (value: boolean) => void;
}) {
  return (
    <>
      <p>{copy.missLead}</p>
      <label style={field}>
        {chrome.llmBaseUrl}
        <input style={control} value={url} onChange={(ev) => onUrl(ev.target.value)} placeholder="https://api.openai.com/v1" />
      </label>
      <label style={field}>
        {chrome.llmModel}
        <LlmModelField t={chrome} baseUrl={url} apiKey={apiKey} model={model} onModel={onModel} inputId="wizard-llm-model" />
      </label>
      <label style={field}>
        {chrome.llmApiKey}
        <input style={control} type="password" value={apiKey} onChange={(ev) => onKey(ev.target.value)} autoComplete="off" />
      </label>
      <label className="wizard-check">
        <input type="checkbox" checked={thinking} onChange={(ev) => onThinking(ev.target.checked)} />
        {chrome.llmThinking}
      </label>
      <p className="caption">{copy.llmSkip}</p>
      <p className="caption" style={{ color: "var(--danger)" }}>{copy.missWarn}</p>
    </>
  );
}
