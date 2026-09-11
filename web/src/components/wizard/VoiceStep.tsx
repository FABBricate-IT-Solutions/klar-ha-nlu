import type { LlmChatTarget } from "../../customVoice";
import { CustomVoiceInterview } from "../CustomVoiceInterview";
import { SearchSelect } from "../SearchSelect";
import { isPersonality, PERSONALITIES, personalityLabel } from "../../personality";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import { effectiveRefineBands, type Settings } from "../../types";
import { SettingsToggle } from "../SettingsToggle";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";

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
        <SettingsToggle
          id="wizard-custom-voice"
          label={chrome.customVoice}
          description={chrome.customVoiceHint}
          checked={customOn}
          onCheckedChange={(checked) => onSettings({ ...settings, personality: checked ? "custom" : "default" })}
        />
      ) : null}
      {customOn && llmReady ? (
        <CustomVoiceInterview t={chrome} language={language} settings={settings} llm={llm} onSettings={onSettings} />
      ) : null}
      {llmReady ? (
        <SettingsToggle
          id="wizard-refine-speech"
          label={chrome.refineSpeech}
          description={chrome.refineSpeechHint}
          checked={Boolean(settings.refine_speech)}
          onCheckedChange={(on) => onSettings({
            ...settings,
            refine_speech: on,
            refine_bands: on && effectiveRefineBands(settings).length === 0 ? ["status"] : settings.refine_bands,
          })}
        />
      ) : null}
    </FieldGroup>
  );
}
