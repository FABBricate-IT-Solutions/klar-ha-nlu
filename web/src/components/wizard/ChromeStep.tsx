import { Moon, Sun } from "lucide-react";
import { languageOptions, SearchSelect, withCurrent } from "../SearchSelect";
import type { LanguagePack } from "../../api";
import type { Messages } from "../../i18n";
import { dictionaries } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Locale, Settings, Theme } from "../../types";
import { SettingsToggle } from "../SettingsToggle";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
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
  const localePacks: { code: string; native_name: string; tier?: string }[] = packs.length
    ? packs.filter((pack) => chromeCodes.has(pack.code))
    : [...chromeCodes].map((code) => ({
      code,
      native_name: code === "de" ? "Deutsch" : code === "en" ? "English" : code,
    }));
  const inUse = new Set(
    [...settings.languages, settings.default_language]
      .map((code) => code.trim().toLowerCase())
      .filter(Boolean),
  );
  const source = packs.length ? packs : localePacks;
  const assistPacks = source.filter((pack) => {
    if (pack.tier !== "experimental") return true;
    if (settings.experimental_languages) return true;
    return inUse.has(pack.code.toLowerCase());
  });
  const tierHint = (tier: string) => {
    if (tier === "experimental") return chrome.languageTierExperimental;
    if (tier === "stable") return chrome.languageTierStable;
    return tier;
  };
  const localeOptions = languageOptions(localePacks, locale, tierHint);
  const assistOptions = languageOptions(assistPacks, locale, tierHint);
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
      <SettingsToggle
        id="wizard-experimental-languages"
        label={chrome.experimentalLanguages}
        description={chrome.experimentalLanguagesHint}
        checked={settings.experimental_languages}
        onCheckedChange={(checked) => onSettings({
          ...settings,
          experimental_languages: checked,
        })}
      />
      <SettingsToggle
        id="wizard-all-assist-languages"
        label={chrome.allAssistLanguages}
        description={chrome.languageHint}
        checked={allAssist}
        onCheckedChange={(checked) => onSettings({
          ...settings,
          languages: checked ? [] : [pinned],
        })}
      />
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
      <Field>
        <FieldLabel>{chrome.defaultLanguage}</FieldLabel>
        <SearchSelect
          value={settings.default_language || ""}
          options={withCurrent(assistOptions, settings.default_language || "")}
          onChange={(value) => onSettings({ ...settings, default_language: value })}
          allowEmpty
          emptyLabel={chrome.defaultLanguageNone}
          placeholder={chrome.languageSearch}
        />
      </Field>
    </FieldGroup>
  );
}
