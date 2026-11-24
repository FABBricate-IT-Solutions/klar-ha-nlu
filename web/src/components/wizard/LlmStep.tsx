import { LlmProviderFields } from "../LlmProviderFields";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { LlmForm } from "../../useLlmForm";
import { FieldGroup } from "@/components/ui/field";

export function LlmStep({ copy, chrome, form }: { copy: WizardMessages; chrome: Messages; form: LlmForm }) {
  return (
    <FieldGroup className="gap-3">
      <p>{copy.missLead}</p>
      <LlmProviderFields t={chrome} form={form} idPrefix="wizard-" modelInputId="wizard-llm-model" />
      <p className="caption">{copy.llmSkip}</p>
      <p className="caption" style={{ color: "var(--danger)" }}>{copy.missWarn}</p>
    </FieldGroup>
  );
}
