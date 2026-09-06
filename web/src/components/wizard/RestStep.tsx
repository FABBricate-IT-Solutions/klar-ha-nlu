import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { field, control } from "./styles";

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
    <>
      <p>{copy.restLead}</p>
      <label style={field}>
        {chrome.mode}
        <select
          style={control}
          value={settings.mode}
          onChange={(ev) => {
            if (ev.target.value === "full" || ev.target.value === "context_only") {
              onSettings({ ...settings, mode: ev.target.value });
            }
          }}
        >
          <option value="full">{chrome.modeFull}</option>
          <option value="context_only">{chrome.modeContext}</option>
        </select>
      </label>
      <label style={field}>
        {chrome.extraPrompt}
        <textarea
          style={{ ...control, minHeight: 88 }}
          value={settings.extra_prompt || ""}
          onChange={(ev) => onSettings({ ...settings, extra_prompt: ev.target.value })}
        />
      </label>
      <label className="wizard-check">
        <input
          type="checkbox"
          checked={settings.confirm_risky_actions}
          onChange={(ev) => onSettings({ ...settings, confirm_risky_actions: ev.target.checked })}
        />
        {chrome.confirmRisky}
      </label>
      <label className="wizard-check">
        <input
          type="checkbox"
          checked={Boolean(settings.quiet_ack)}
          onChange={(ev) => onSettings({ ...settings, quiet_ack: ev.target.checked })}
        />
        {chrome.quietAck}
      </label>
      <label className="wizard-check">
        <input
          type="checkbox"
          checked={Boolean(settings.nlu_rag)}
          onChange={(ev) => onSettings({ ...settings, nlu_rag: ev.target.checked })}
        />
        {chrome.nluRag}
      </label>
      {llmReady ? (
        <>
          <label className="wizard-check">
            <input
              type="checkbox"
              checked={Boolean(settings.calendar_llm)}
              onChange={(ev) => onSettings({ ...settings, calendar_llm: ev.target.checked })}
            />
            {chrome.calendarLlm}
          </label>
          <label className="wizard-check">
            <input
              type="checkbox"
              checked={Boolean(settings.allow_llm_tools)}
              onChange={(ev) => onSettings({ ...settings, allow_llm_tools: ev.target.checked })}
            />
            {chrome.allowLlmTools}
          </label>
        </>
      ) : null}
    </>
  );
}
