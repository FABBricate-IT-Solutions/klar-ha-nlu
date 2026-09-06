import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
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
      </Field>
      <Field>
        <FieldLabel htmlFor="wizard-extra-prompt">{chrome.extraPrompt}</FieldLabel>
        <Textarea
          id="wizard-extra-prompt"
          value={settings.extra_prompt || ""}
          onChange={(ev) => onSettings({ ...settings, extra_prompt: ev.target.value })}
        />
      </Field>
      <Field orientation="horizontal">
        <FieldContent>
          <FieldLabel>{chrome.confirmRisky}</FieldLabel>
        </FieldContent>
        <Switch
          checked={settings.confirm_risky_actions}
          onCheckedChange={(checked) => onSettings({ ...settings, confirm_risky_actions: Boolean(checked) })}
        />
      </Field>
      <Field orientation="horizontal">
        <FieldContent>
          <FieldLabel>{chrome.quietAck}</FieldLabel>
        </FieldContent>
        <Switch
          checked={Boolean(settings.quiet_ack)}
          onCheckedChange={(checked) => onSettings({ ...settings, quiet_ack: Boolean(checked) })}
        />
      </Field>
      <Field orientation="horizontal">
        <FieldContent>
          <FieldLabel>{chrome.nluRag}</FieldLabel>
        </FieldContent>
        <Switch
          checked={Boolean(settings.nlu_rag)}
          onCheckedChange={(checked) => onSettings({ ...settings, nlu_rag: Boolean(checked) })}
        />
      </Field>
      {llmReady ? (
        <>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{chrome.calendarLlm}</FieldLabel>
            </FieldContent>
            <Switch
              checked={Boolean(settings.calendar_llm)}
              onCheckedChange={(checked) => onSettings({ ...settings, calendar_llm: Boolean(checked) })}
            />
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{chrome.allowLlmTools}</FieldLabel>
              <FieldDescription>{chrome.allowLlmToolsHint}</FieldDescription>
            </FieldContent>
            <Switch
              checked={Boolean(settings.allow_llm_tools)}
              onCheckedChange={(checked) => onSettings({ ...settings, allow_llm_tools: Boolean(checked) })}
            />
          </Field>
        </>
      ) : null}
    </FieldGroup>
  );
}
