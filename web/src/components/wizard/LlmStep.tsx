import { LlmProviderFields } from "../LlmProviderFields";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import { FieldGroup } from "@/components/ui/field";

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
    <FieldGroup className="gap-3">
      <p>{copy.missLead}</p>
      <LlmProviderFields
        t={chrome}
        baseUrl={url}
        model={model}
        apiKey={apiKey}
        thinking={thinking}
        onUrl={onUrl}
        onModel={onModel}
        onKey={onKey}
        onThinking={onThinking}
        idPrefix="wizard-"
        modelInputId="wizard-llm-model"
      />
      <p className="caption">{copy.llmSkip}</p>
      <p className="caption" style={{ color: "var(--danger)" }}>{copy.missWarn}</p>
    </FieldGroup>
  );
}
