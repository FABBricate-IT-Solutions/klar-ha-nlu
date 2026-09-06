import { CustomVoiceInterview } from "../CustomVoiceInterview";
import { isPersonality, PERSONALITIES, personalityLabel } from "../../personality";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { field, control } from "./styles";

export function VoiceStep({
  copy,
  chrome,
  settings,
  llmReady,
  language,
  onSettings,
}: {
  copy: WizardMessages;
  chrome: Messages;
  settings: Settings;
  llmReady: boolean;
  language: string;
  onSettings: (next: Settings) => void;
}) {
  const voice = isPersonality(settings.personality) ? settings.personality : "default";
  const customOn = voice === "custom";
  return (
    <>
      <p>{copy.modeLead}</p>
      <label style={field}>
        {chrome.personality}
        <select
          style={control}
          value={voice}
          onChange={(ev) => {
            if (isPersonality(ev.target.value)) {
              onSettings({ ...settings, personality: ev.target.value });
            }
          }}
        >
          {PERSONALITIES.filter((id) => id !== "custom" || llmReady).map((id) => (
            <option key={id} value={id}>{personalityLabel(chrome, id)}</option>
          ))}
        </select>
      </label>
      {llmReady ? (
        <label className="wizard-check">
          <input
            type="checkbox"
            checked={customOn}
            onChange={(ev) => onSettings({ ...settings, personality: ev.target.checked ? "custom" : "default" })}
          />
          {chrome.customVoice}
        </label>
      ) : null}
      {customOn && llmReady ? (
        <CustomVoiceInterview
          t={chrome}
          language={language}
          value={settings.custom_voice || ""}
          onChange={(prompt) => onSettings({ ...settings, personality: "custom", custom_voice: prompt })}
        />
      ) : null}
      {llmReady ? (
        <label className="wizard-check">
          <input
            type="checkbox"
            checked={Boolean(settings.refine_speech)}
            onChange={(ev) => onSettings({ ...settings, refine_speech: ev.target.checked })}
          />
          {chrome.refineSpeech}
        </label>
      ) : null}
      {llmReady ? <p className="caption">{chrome.refineSpeechHint}</p> : null}
    </>
  );
}
