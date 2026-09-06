import { CustomVoiceInterview } from "./CustomVoiceInterview";
import { PersonalityPrompt } from "./PersonalityPrompt";
import { languageOptions, SearchSelect, withCurrent } from "./SearchSelect";
import type { LanguagePack } from "../api";
import { dictionaries, type Messages } from "../i18n";
import { isPersonality, PERSONALITIES, personalityLabel } from "../personality";
import type { BundleList, Locale, Settings, Theme } from "../types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

export function SettingsVoiceSection({
  t,
  settings,
  onSettings,
  locale,
  llmReady,
}: {
  t: Messages;
  settings: Settings;
  onSettings: (s: Settings) => void;
  locale: Locale;
  llmReady: boolean;
}) {
  const pinned = settings.languages[0] || locale;
  const voice = isPersonality(settings.personality) ? settings.personality : "default";
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t.voice}</CardTitle>
        <CardDescription>{t.voiceHint}</CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel>{t.personality}</FieldLabel>
            <SearchSelect
              value={voice}
              options={PERSONALITIES.map((id) => ({
                value: id,
                label: personalityLabel(t, id, settings.custom_voice_name),
              }))}
              onChange={(value) => {
                if (isPersonality(value)) onSettings({ ...settings, personality: value });
              }}
              allowEmpty={false}
              placeholder={t.personality}
            />
          </Field>
          {voice === "custom" ? (
            <CustomVoiceInterview t={t} language={pinned} settings={settings} onSettings={onSettings} />
          ) : (
            <PersonalityPrompt t={t} personality={voice} language={pinned} />
          )}
          {llmReady && voice !== "custom" ? (
            <Field>
              <Button variant="outline" type="button" onClick={() => onSettings({ ...settings, personality: "custom" })}>
                {t.customVoice}
              </Button>
            </Field>
          ) : null}
          <Field>
            <FieldLabel htmlFor="klar-extra-prompt">{t.extraPrompt}</FieldLabel>
            <Textarea
              id="klar-extra-prompt"
              value={settings.extra_prompt || ""}
              onChange={(ev) => onSettings({ ...settings, extra_prompt: ev.target.value })}
            />
            <FieldDescription>{t.extraPromptHint}</FieldDescription>
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.refineSpeech}</FieldLabel>
              <FieldDescription>{t.refineSpeechHint}</FieldDescription>
            </FieldContent>
            <Switch
              checked={Boolean(settings.refine_speech)}
              onCheckedChange={(checked) => onSettings({ ...settings, refine_speech: Boolean(checked) })}
            />
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.quietAck}</FieldLabel>
              <FieldDescription>{t.quietAckHint}</FieldDescription>
            </FieldContent>
            <Switch
              checked={Boolean(settings.quiet_ack)}
              onCheckedChange={(checked) => onSettings({ ...settings, quiet_ack: Boolean(checked) })}
            />
          </Field>
          <Field>
            <FieldLabel>{t.unitSystem}</FieldLabel>
            <ToggleGroup
              variant="outline"
              spacing={0}
              value={[settings.unit_system === "imperial" ? "imperial" : "metric"]}
              onValueChange={(next) => {
                const value = next[0];
                if (value === "metric" || value === "imperial") onSettings({ ...settings, unit_system: value });
              }}
              aria-label={t.unitSystem}
            >
              <ToggleGroupItem value="metric">{t.unitMetric}</ToggleGroupItem>
              <ToggleGroupItem value="imperial">{t.unitImperial}</ToggleGroupItem>
            </ToggleGroup>
            <FieldDescription>{t.unitSystemHint}</FieldDescription>
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}

export function SettingsLanguagesSection({
  t,
  locale,
  onLocale,
  settings,
  onSettings,
  packs,
}: {
  t: Messages;
  locale: Locale;
  onLocale: (locale: Locale) => void;
  settings: Settings;
  onSettings: (s: Settings) => void;
  packs: LanguagePack[];
}) {
  const chromeCodes = new Set(Object.keys(dictionaries));
  const localePacks = packs.length
    ? packs.filter((pack) => chromeCodes.has(pack.code))
    : [...chromeCodes].map((code) => ({ code, native_name: code }));
  const localeOptions = languageOptions(localePacks, locale);
  const assistOptions = languageOptions(packs.length ? packs : localePacks, locale);
  const allAssist = settings.languages.length === 0;
  const pinned = settings.languages[0] || locale;
  return (
    <div className="grid min-w-0 gap-4">
      <Card>
        <CardHeader>
          <CardTitle>{t.assistLanguages}</CardTitle>
          <CardDescription>{t.assistLanguagesHint}</CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel>{t.allAssistLanguages}</FieldLabel>
                <FieldDescription>{t.languageHint}</FieldDescription>
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
                <FieldLabel>{t.pinLanguage}</FieldLabel>
                <SearchSelect
                  value={pinned}
                  options={withCurrent(assistOptions, pinned)}
                  onChange={(value) => onSettings({ ...settings, languages: value ? [value] : [] })}
                  allowEmpty={false}
                  placeholder={t.languageSearch}
                />
              </Field>
            )}
          </FieldGroup>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>{t.operatorLanguage}</CardTitle>
          <CardDescription>{t.operatorLanguageHint}</CardDescription>
        </CardHeader>
        <CardContent>
          <Field>
            <FieldLabel>{t.operatorLanguage}</FieldLabel>
            <SearchSelect
              value={locale}
              options={withCurrent(localeOptions, locale)}
              onChange={onLocale}
              allowEmpty={false}
              placeholder={t.languageSearch}
            />
          </Field>
        </CardContent>
      </Card>
    </div>
  );
}

export function SettingsEngineSection({
  t,
  settings,
  onSettings,
  token,
  onToken,
  theme,
  onTheme,
}: {
  t: Messages;
  settings: Settings;
  onSettings: (s: Settings) => void;
  token: string;
  onToken: (value: string) => void;
  theme: Theme;
  onTheme: (theme: Theme) => void;
}) {
  return (
    <div className="grid min-w-0 gap-4">
      <Card>
        <CardHeader>
          <CardTitle>{t.missTitle}</CardTitle>
          <CardDescription>{t.missHint}</CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field>
              <FieldLabel>{t.mode}</FieldLabel>
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
                aria-label={t.mode}
              >
                <ToggleGroupItem value="full">{t.modeFull}</ToggleGroupItem>
                <ToggleGroupItem value="context_only">{t.modeContext}</ToggleGroupItem>
              </ToggleGroup>
            </Field>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel>{t.confirmRisky}</FieldLabel>
              </FieldContent>
              <Switch
                checked={settings.confirm_risky_actions}
                onCheckedChange={(checked) => onSettings({ ...settings, confirm_risky_actions: Boolean(checked) })}
              />
            </Field>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel>{t.nluRag}</FieldLabel>
                <FieldDescription>{t.nluRagHint}</FieldDescription>
              </FieldContent>
              <Switch
                checked={settings.nlu_rag}
                onCheckedChange={(checked) => onSettings({ ...settings, nlu_rag: Boolean(checked) })}
              />
            </Field>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel>{t.calendarLlm}</FieldLabel>
                <FieldDescription>{t.calendarLlmHint}</FieldDescription>
              </FieldContent>
              <Switch
                checked={Boolean(settings.calendar_llm)}
                onCheckedChange={(checked) => onSettings({ ...settings, calendar_llm: Boolean(checked) })}
              />
            </Field>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel>{t.allowLlmTools}</FieldLabel>
                <FieldDescription>{t.allowLlmToolsHint}</FieldDescription>
              </FieldContent>
              <Switch
                checked={Boolean(settings.allow_llm_tools)}
                onCheckedChange={(checked) => onSettings({ ...settings, allow_llm_tools: Boolean(checked) })}
              />
            </Field>
          </FieldGroup>
        </CardContent>
      </Card>
      <Card>
        <CardHeader>
          <CardTitle>{t.operatorChrome}</CardTitle>
          <CardDescription>{t.operatorChromeHint}</CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field>
              <FieldLabel>{t.operatorChrome}</FieldLabel>
              <ToggleGroup
                variant="outline"
                spacing={0}
                value={[theme]}
                onValueChange={(next) => {
                  const value = next[0];
                  if (value === "dark" || value === "light") onTheme(value);
                }}
                aria-label={t.operatorChrome}
              >
                <ToggleGroupItem value="dark">{t.appearanceDark}</ToggleGroupItem>
                <ToggleGroupItem value="light">{t.appearanceLight}</ToggleGroupItem>
              </ToggleGroup>
            </Field>
            <Field>
              <FieldLabel htmlFor="klar-token">{t.token}</FieldLabel>
              <Input id="klar-token" type="password" value={token} onChange={(ev) => onToken(ev.target.value)} />
            </Field>
          </FieldGroup>
        </CardContent>
      </Card>
    </div>
  );
}

export function SettingsJournalSection({
  t,
  settings,
  bundle,
  onToggle,
  onDownloadDataset,
  onDownloadProtocol,
  onClear,
}: {
  t: Messages;
  settings: Settings;
  bundle: BundleList | null;
  onToggle: (next: Settings) => void;
  onDownloadDataset: () => void;
  onDownloadProtocol: () => void;
  onClear: () => void;
}) {
  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>{t.supportBundle}</CardTitle>
        <CardDescription>{t.journalHint}</CardDescription>
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.recordProtocol}</FieldLabel>
            </FieldContent>
            <Switch
              checked={settings.support_bundle}
              onCheckedChange={(checked) => onToggle({ ...settings, support_bundle: Boolean(checked) })}
            />
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.includeRawText}</FieldLabel>
            </FieldContent>
            <Switch
              checked={settings.support_bundle_raw_text}
              onCheckedChange={(checked) => onToggle({ ...settings, support_bundle_raw_text: Boolean(checked) })}
            />
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.semanticAdapters}</FieldLabel>
            </FieldContent>
            <Switch
              checked={settings.semantic_adapters}
              onCheckedChange={(checked) => onToggle({ ...settings, semantic_adapters: Boolean(checked) })}
            />
          </Field>
          <Field>
            <FieldLabel>{t.journal}</FieldLabel>
            <FieldDescription>{bundle ? `${bundle.count} ${t.recordings}` : "..."}</FieldDescription>
          </Field>
        </FieldGroup>
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        <Button variant="outline" type="button" onClick={onDownloadDataset}>{t.downloadDataset}</Button>
        <Button variant="outline" type="button" onClick={onDownloadProtocol}>{t.downloadProtocol}</Button>
        <Button variant="destructive" type="button" onClick={onClear}>{t.clearAll}</Button>
      </CardFooter>
    </Card>
  );
}
