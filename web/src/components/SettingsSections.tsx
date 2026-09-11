import { CustomVoiceInterview } from "./CustomVoiceInterview";
import { PersonalityPrompt } from "./PersonalityPrompt";
import { languageOptions, SearchSelect, withCurrent } from "./SearchSelect";
import { SettingsToggle } from "./SettingsToggle";
import type { LanguagePack } from "../api";
import { dictionaries, type Messages } from "../i18n";
import { isPersonality, PERSONALITIES, personalityLabel } from "../personality";
import { REFINE_BANDS, effectiveRefineBands, type BundleList, type Locale, type RefineBand, type Settings, type Theme } from "../types";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

function refineBandLabel(t: Messages, band: RefineBand): string {
  switch (band) {
    case "status":
      return t.refineBandStatus;
    case "command":
      return t.refineBandCommand;
    case "prompt":
      return t.refineBandPrompt;
    case "reject":
      return t.refineBandReject;
    default: {
      const exhaustive: never = band;
      return exhaustive;
    }
  }
}

function refineBandHint(t: Messages, band: RefineBand): string {
  switch (band) {
    case "status":
      return t.refineBandStatusHint;
    case "command":
      return t.refineBandCommandHint;
    case "prompt":
      return t.refineBandPromptHint;
    case "reject":
      return t.refineBandRejectHint;
    default: {
      const exhaustive: never = band;
      return exhaustive;
    }
  }
}

function toggleRefineBand(settings: Settings, band: RefineBand, on: boolean): RefineBand[] {
  const current = effectiveRefineBands(settings);
  if (on) return current.includes(band) ? current : [...current, band];
  return current.filter((item) => item !== band);
}

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
          <SettingsToggle
            id="klar-refine-speech"
            label={t.refineSpeech}
            description={t.refineSpeechHint}
            checked={Boolean(settings.refine_speech)}
            onCheckedChange={(on) => onSettings({
              ...settings,
              refine_speech: on,
              refine_bands: on && effectiveRefineBands(settings).length === 0 ? ["status"] : settings.refine_bands,
            })}
          />
          {settings.refine_speech ? (
            <div className="flex flex-col gap-5 border-l border-border pl-4">
              {REFINE_BANDS.map((band) => (
                <SettingsToggle
                  key={band}
                  id={`klar-refine-band-${band}`}
                  label={refineBandLabel(t, band)}
                  description={refineBandHint(t, band)}
                  checked={effectiveRefineBands(settings).includes(band)}
                  onCheckedChange={(checked) => onSettings({ ...settings, refine_bands: toggleRefineBand(settings, band, checked) })}
                />
              ))}
              <FieldDescription>{t.refineBandsHint}</FieldDescription>
            </div>
          ) : null}
          <SettingsToggle
            id="klar-quiet-ack"
            label={t.quietAck}
            description={t.quietAckHint}
            checked={Boolean(settings.quiet_ack)}
            onCheckedChange={(checked) => onSettings({ ...settings, quiet_ack: checked })}
          />
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
            <SettingsToggle
              id="klar-all-assist-languages"
              label={t.allAssistLanguages}
              description={t.languageHint}
              checked={allAssist}
              onCheckedChange={(checked) => onSettings({
                ...settings,
                languages: checked ? [] : [pinned],
              })}
            />
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
              <FieldDescription>{t.modeHint}</FieldDescription>
            </Field>
            <SettingsToggle
              id="klar-confirm-risky"
              label={t.confirmRisky}
              description={t.confirmRiskyHint}
              checked={settings.confirm_risky_actions}
              onCheckedChange={(checked) => onSettings({ ...settings, confirm_risky_actions: checked })}
            />
            <SettingsToggle
              id="klar-nlu-rag"
              label={t.nluRag}
              description={t.nluRagHint}
              checked={settings.nlu_rag}
              onCheckedChange={(checked) => onSettings({ ...settings, nlu_rag: checked })}
            />
            <SettingsToggle
              id="klar-semantic-adapters"
              label={t.semanticAdapters}
              description={t.semanticAdaptersHint}
              checked={settings.semantic_adapters}
              onCheckedChange={(checked) => onSettings({ ...settings, semantic_adapters: checked })}
            />
            <SettingsToggle
              id="klar-calendar-llm"
              label={t.calendarLlm}
              description={t.calendarLlmHint}
              checked={Boolean(settings.calendar_llm)}
              onCheckedChange={(checked) => onSettings({ ...settings, calendar_llm: checked })}
            />
            <SettingsToggle
              id="klar-allow-llm-tools"
              label={t.allowLlmTools}
              description={t.allowLlmToolsHint}
              checked={Boolean(settings.allow_llm_tools)}
              onCheckedChange={(checked) => onSettings({ ...settings, allow_llm_tools: checked })}
            />
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
              <FieldDescription>{t.tokenHint}</FieldDescription>
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
          <SettingsToggle
            id="klar-record-protocol"
            label={t.recordProtocol}
            description={t.recordProtocolHint}
            checked={settings.support_bundle}
            onCheckedChange={(checked) => onToggle({ ...settings, support_bundle: checked })}
          />
          <SettingsToggle
            id="klar-include-raw-text"
            label={t.includeRawText}
            description={t.includeRawTextHint}
            checked={settings.support_bundle_raw_text}
            onCheckedChange={(checked) => onToggle({ ...settings, support_bundle_raw_text: checked })}
          />
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
