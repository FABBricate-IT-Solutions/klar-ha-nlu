import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { SettingsToggle } from "../SettingsToggle";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

export function RestStep({
  copy,
  chrome,
  settings,
  llmReady,
  onSettings,
}: {
  copy: WizardMessages;
  chrome: Messages;
  settings: Settings;
  llmReady: boolean;
  onSettings: (next: Settings) => void;
}) {
  return (
    <FieldGroup>
      <p>{copy.restLead}</p>
      <Field>
        <FieldLabel>{chrome.mode}</FieldLabel>
        <ToggleGroup
          variant="outline"
          spacing={0}
          value={[settings.mode]}
          onValueChange={(next) => {
            const value = next[0];
            if (value === "full" || value === "context_only") {
              onSettings({ ...settings, mode: value });
            }
          }}
          aria-label={chrome.mode}
        >
          <ToggleGroupItem value="full">{chrome.modeFull}</ToggleGroupItem>
          <ToggleGroupItem value="context_only">{chrome.modeContext}</ToggleGroupItem>
        </ToggleGroup>
        <FieldDescription>{chrome.modeHint}</FieldDescription>
      </Field>
      <Field>
        <FieldLabel htmlFor="wizard-extra-prompt">{chrome.extraPrompt}</FieldLabel>
        <Textarea
          id="wizard-extra-prompt"
          value={settings.extra_prompt || ""}
          onChange={(ev) => onSettings({ ...settings, extra_prompt: ev.target.value })}
        />
        <FieldDescription>{chrome.extraPromptHint}</FieldDescription>
      </Field>
      <SettingsToggle
        id="wizard-confirm-risky"
        label={chrome.confirmRisky}
        description={chrome.confirmRiskyHint}
        checked={settings.confirm_risky_actions}
        onCheckedChange={(checked) => onSettings({ ...settings, confirm_risky_actions: checked })}
      />
      <SettingsToggle
        id="wizard-quiet-ack"
        label={chrome.quietAck}
        description={chrome.quietAckHint}
        checked={Boolean(settings.quiet_ack)}
        onCheckedChange={(checked) => onSettings({ ...settings, quiet_ack: checked })}
      />
      <SettingsToggle
        id="wizard-nlu-rag"
        label={chrome.nluRag}
        description={chrome.nluRagHint}
        checked={Boolean(settings.nlu_rag)}
        onCheckedChange={(checked) => onSettings({ ...settings, nlu_rag: checked })}
      />
      {llmReady ? (
        <>
          <SettingsToggle
            id="wizard-calendar-llm"
            label={chrome.calendarLlm}
            description={chrome.calendarLlmHint}
            checked={Boolean(settings.calendar_llm)}
            onCheckedChange={(checked) => onSettings({ ...settings, calendar_llm: checked })}
          />
          <SettingsToggle
            id="wizard-allow-llm-tools"
            label={chrome.allowLlmTools}
            description={chrome.allowLlmToolsHint}
            checked={Boolean(settings.allow_llm_tools)}
            onCheckedChange={(checked) => onSettings({ ...settings, allow_llm_tools: checked })}
          />
        </>
      ) : null}
    </FieldGroup>
  );
}
