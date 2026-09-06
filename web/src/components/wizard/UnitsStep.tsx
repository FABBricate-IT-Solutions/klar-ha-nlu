import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Settings } from "../../types";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

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
    <FieldGroup>
      <p>{copy.unitsLead}</p>
      <Field>
        <FieldLabel>{chrome.unitSystem}</FieldLabel>
        <ToggleGroup
          variant="outline"
          spacing={0}
          value={[unit]}
          onValueChange={(next) => {
            const value = next[0];
            if (value === "metric" || value === "imperial") {
              onSettings({ ...settings, unit_system: value });
            }
          }}
          aria-label={chrome.unitSystem}
        >
          <ToggleGroupItem value="metric">{chrome.unitMetric}</ToggleGroupItem>
          <ToggleGroupItem value="imperial">{chrome.unitImperial}</ToggleGroupItem>
        </ToggleGroup>
        <FieldDescription>{chrome.unitSystemHint}</FieldDescription>
      </Field>
    </FieldGroup>
  );
}
