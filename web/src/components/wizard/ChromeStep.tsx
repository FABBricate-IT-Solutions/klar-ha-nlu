import { Moon, Sun } from "lucide-react";
import { SearchSelect, withCurrent } from "../SearchSelect";
import type { LanguagePack } from "../../api";
import type { Messages } from "../../i18n";
import type { WizardMessages } from "../../i18n/wizard";
import type { Locale, Settings, Theme } from "../../types";
import { dictionaries } from "../../i18n";
import { field } from "./styles";

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
  const localeOptions = (packs.length ? packs.filter((pack) => chromeCodes.has(pack.code)) : [...chromeCodes].map((code) => ({
    code,
    native_name: code,
    script: "",
    variants: [code],
  }))).map((pack) => ({ value: pack.code, label: `${pack.native_name} (${pack.code})` }));
  const assistOptions = (packs.length ? packs : localeOptions.map((row) => ({
    code: row.value,
    native_name: row.label,
    script: "",
    variants: [row.value],
  }))).map((pack) => ({ value: pack.code, label: `${pack.native_name} (${pack.code})` }));
  const allAssist = settings.languages.length === 0;
  const pinned = settings.languages[0] || locale;
  return (
    <>
      <p>{copy.chromeLead}</p>
      <div className="wizard-theme">
        <button
          type="button"
          className={theme === "dark" ? "primary" : "secondary"}
          aria-label={chrome.appearanceDark}
          aria-pressed={theme === "dark"}
          onClick={() => onTheme("dark")}
        >
          <Moon />
        </button>
        <button
          type="button"
          className={theme === "light" ? "primary" : "secondary"}
          aria-label={chrome.appearanceLight}
          aria-pressed={theme === "light"}
          onClick={() => onTheme("light")}
        >
          <Sun />
        </button>
      </div>
      <label style={field}>
        {chrome.operatorLanguage}
        <div style={{ marginTop: 6 }}>
          <SearchSelect
            value={locale}
            options={withCurrent(localeOptions, locale)}
            onChange={(value) => onLocale(value)}
            allowEmpty={false}
            placeholder={chrome.languageSearch}
          />
        </div>
      </label>
      <label className="wizard-check">
        <input
          type="checkbox"
          checked={allAssist}
          onChange={(ev) => onSettings({
            ...settings,
            languages: ev.target.checked ? [] : [pinned],
          })}
        />
        {chrome.allAssistLanguages}
      </label>
      {allAssist ? null : (
        <label style={field}>
          {chrome.pinLanguage}
          <div style={{ marginTop: 6 }}>
            <SearchSelect
              value={pinned}
              options={withCurrent(assistOptions, pinned)}
              onChange={(value) => onSettings({ ...settings, languages: value ? [value] : [] })}
              allowEmpty={false}
              placeholder={chrome.languageSearch}
            />
          </div>
        </label>
      )}
    </>
  );
}
