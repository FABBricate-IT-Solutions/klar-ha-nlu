import type { LlmChatTarget } from "../../customVoice";
import { CustomVoiceInterview } from "../CustomVoiceInterview";
import { SearchSelect } from "../SearchSelect";
import { isPersonality, PERSONALITIES, personalityLabel } from "../../personality";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";

export function VoiceStep({
  copy,
  chrome,
  settings,
  llmReady,
  language,
  llm,
  onSettings,
}: {
  copy: WizardMessages;
  chrome: Messages;
  settings: Settings;
  llmReady: boolean;
  language: string;
  llm?: LlmChatTarget;
  onSettings: (next: Settings) => void;
}) {
  const voice = isPersonality(settings.personality) ? settings.personality : "default";
  const customOn = voice === "custom";
  const voices = PERSONALITIES.filter((id) => id !== "custom" || llmReady).map((id) => ({
    value: id,
    label: personalityLabel(chrome, id, settings.custom_voice_name),
  }));
  return (
    <FieldGroup>
      <p>{copy.modeLead}</p>
      <Field>
        <FieldLabel>{chrome.personality}</FieldLabel>
        <SearchSelect
          value={voice}
          options={voices}
          onChange={(value) => {
            if (isPersonality(value)) onSettings({ ...settings, personality: value });
          }}
          allowEmpty={false}
          placeholder={chrome.personality}
        />
      </Field>
      {llmReady ? (
        <Field orientation="horizontal">
          <FieldContent>
            <FieldLabel>{chrome.customVoice}</FieldLabel>
          </FieldContent>
          <Switch
            checked={customOn}
            onCheckedChange={(checked) => onSettings({ ...settings, personality: checked ? "custom" : "default" })}
          />
        </Field>
      ) : null}
      {customOn && llmReady ? (
        <CustomVoiceInterview t={chrome} language={language} settings={settings} llm={llm} onSettings={onSettings} />
      ) : null}
      {llmReady ? (
        <Field orientation="horizontal">
          <FieldContent>
            <FieldLabel>{chrome.refineSpeech}</FieldLabel>
            <FieldDescription>{chrome.refineSpeechHint}</FieldDescription>
          </FieldContent>
          <Switch
            checked={Boolean(settings.refine_speech)}
            onCheckedChange={(checked) => onSettings({ ...settings, refine_speech: Boolean(checked) })}
          />
        </Field>
      ) : null}
    </FieldGroup>
  );
}
