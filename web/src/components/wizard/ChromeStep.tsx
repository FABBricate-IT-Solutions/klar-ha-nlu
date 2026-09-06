import { Moon, Sun } from "lucide-react";
import { languageOptions, SearchSelect, withCurrent } from "../SearchSelect";
import type { LanguagePack } from "../../api";
import type { Messages } from "../../i18n";
import { dictionaries } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Locale, Settings, Theme } from "../../types";
import { Field, FieldContent, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

export function ChromeStep({
  copy,
  chrome,
  locale,
  theme,
  settings,
  packs,
  onLocale,
  onTheme,
  onSettings,
}: {
  copy: WizardMessages;
  chrome: Messages;
  locale: Locale;
  theme: Theme;
  settings: Settings;
  packs: LanguagePack[];
  onLocale: (locale: Locale) => void;
  onTheme: (theme: Theme) => void;
  onSettings: (next: Settings) => void;
}) {
  const chromeCodes = new Set(Object.keys(dictionaries));
  const localePacks = packs.length
    ? packs.filter((pack) => chromeCodes.has(pack.code))
    : [...chromeCodes].map((code) => ({
      code,
      native_name: code === "de" ? "Deutsch" : code === "en" ? "English" : code,
    }));
  const assistPacks = packs.length ? packs : localePacks;
  const localeOptions = languageOptions(localePacks, locale);
  const assistOptions = languageOptions(assistPacks, locale);
  const allAssist = settings.languages.length === 0;
  const pinned = settings.languages[0] || locale;
  return (
    <FieldGroup>
      <p>{copy.chromeLead}</p>
      <Field>
        <FieldLabel>{chrome.operatorChrome}</FieldLabel>
        <ToggleGroup
          className="wizard-theme"
          variant="outline"
          spacing={2}
          value={[theme]}
          onValueChange={(next) => {
            const value = next[0];
            if (value === "dark" || value === "light") onTheme(value);
          }}
          aria-label={chrome.operatorChrome}
        >
          <ToggleGroupItem value="dark" aria-label={chrome.appearanceDark} className="size-[72px] p-0 [&_svg]:size-6">
            <Moon />
          </ToggleGroupItem>
          <ToggleGroupItem value="light" aria-label={chrome.appearanceLight} className="size-[72px] p-0 [&_svg]:size-6">
            <Sun />
          </ToggleGroupItem>
        </ToggleGroup>
      </Field>
      <Field>
        <FieldLabel>{chrome.operatorLanguage}</FieldLabel>
        <SearchSelect
          value={locale}
          options={withCurrent(localeOptions, locale)}
          onChange={(value) => onLocale(value)}
          allowEmpty={false}
          placeholder={chrome.languageSearch}
        />
      </Field>
      <Field orientation="horizontal">
        <FieldContent>
          <FieldLabel>{chrome.allAssistLanguages}</FieldLabel>
        </FieldContent>
        <Switch
          checked={allAssist}
          onCheckedChange={(checked) => onSettings({
            ...settings,
            languages: checked ? [] : [pinned],
          })}
        />
      </Field>
      {allAssist ? null : (
        <Field>
          <FieldLabel>{chrome.pinLanguage}</FieldLabel>
          <SearchSelect
            value={pinned}
            options={withCurrent(assistOptions, pinned)}
            onChange={(value) => onSettings({ ...settings, languages: value ? [value] : [] })}
            allowEmpty={false}
            placeholder={chrome.languageSearch}
          />
        </Field>
      )}
    </FieldGroup>
  );
}
