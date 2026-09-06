import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { field, control } from "./styles";

export function UnitsStep({
  copy,
  chrome,
  settings,
  onSettings,
}: {
  copy: WizardMessages;
  chrome: Messages;
  settings: Settings;
  onSettings: (next: Settings) => void;
}) {
  const unit = settings.unit_system === "imperial" ? "imperial" : "metric";
  return (
    <>
      <p>{copy.unitsLead}</p>
      <label style={field}>
        {chrome.unitSystem}
        <select
          style={control}
          value={unit}
          onChange={(ev) => {
            if (ev.target.value === "metric" || ev.target.value === "imperial") {
              onSettings({ ...settings, unit_system: ev.target.value });
            }
          }}
        >
          <option value="metric">{chrome.unitMetric}</option>
          <option value="imperial">{chrome.unitImperial}</option>
        </select>
      </label>
      <p className="caption">{chrome.unitSystemHint}</p>
    </>
  );
}
