import { useState } from "react";
import { CustomVoiceInterview } from "./CustomVoiceInterview";
import { PersonalityPrompt } from "./PersonalityPrompt";
import { languageOptions, SearchSelect, withCurrent } from "./SearchSelect";
import { SettingsToggle } from "./SettingsToggle";
import type { LanguagePack } from "../api";
import { fill, localeCodes, plural, type Messages } from "../i18n";
import { isPersonality, PERSONALITIES, personalityLabel } from "../personality";
import {
  REFINE_BANDS,
  effectiveRefineBands,
  type BundleList,
  type LanguageTier,
  type Locale,
  type RefineBand,
  type Settings,
  type Theme,
} from "../types";
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
          <SettingsToggle
            id="klar-satellite-followup"
            label={t.satelliteFollowup}
            description={t.satelliteFollowupHint}
            checked={Boolean(settings.satellite_followup)}
            onCheckedChange={(checked) => onSettings({ ...settings, satellite_followup: checked })}
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

function tierLabel(t: Messages, tier: string): string {
  switch (tier) {
    case "stable":
      return t.languageTierStable;
    case "experimental":
      return t.languageTierExperimental;
    default:
      return tier;
  }
}

type PackRow = { code: string; native_name: string; tier?: LanguageTier };

/** Locales the draft settings may pick: stable always, experimental with the switch, and whatever is already in use. */
function selectablePacks(packs: PackRow[], settings: Settings): PackRow[] {
  const inUse = new Set(
    [...settings.languages, settings.default_language]
      .map((code) => code.trim().toLowerCase())
      .filter(Boolean),
  );
  return packs.filter((pack) => {
    if (pack.tier !== "experimental") return true;
    if (settings.experimental_languages) return true;
    return inUse.has(pack.code.toLowerCase());
  });
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
  const chromeCodes = new Set(localeCodes);
  const localePacks = packs.length
    ? packs.filter((pack) => chromeCodes.has(pack.code))
    : [...chromeCodes].map((code) => ({ code, native_name: code }));
  const labelTier = (tier: string) => tierLabel(t, tier);
  const localeOptions = languageOptions(localePacks, locale, labelTier);
  const offered = selectablePacks(packs.length ? packs : localePacks, settings);
  const assistOptions = languageOptions(offered, locale, labelTier);
  const defaultOptions = languageOptions(offered, locale, labelTier);
  const allAssist = settings.languages.length === 0;
  const pinned = settings.languages[0] || locale;
  const defaultPack = settings.default_language || "";
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
              id="klar-experimental-languages"
              label={t.experimentalLanguages}
              description={t.experimentalLanguagesHint}
              checked={settings.experimental_languages}
              onCheckedChange={(checked) => onSettings({
                ...settings,
                experimental_languages: checked,
              })}
            />
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
                  onChange={(value) => {
                    // Clearing the box must not flip Assist to "every locale" (U-24).
                    if (!value) return;
                    onSettings({ ...settings, languages: [value] });
                  }}
                  allowEmpty={false}
                  placeholder={t.languageSearch}
                />
              </Field>
            )}
            <Field>
              <FieldLabel>{t.defaultLanguage}</FieldLabel>
              <FieldDescription>{t.defaultLanguageHint}</FieldDescription>
              <SearchSelect
                value={defaultPack}
                options={withCurrent(defaultOptions, defaultPack)}
                onChange={(value) => onSettings({ ...settings, default_language: value })}
                allowEmpty
                emptyLabel={t.defaultLanguageNone}
                placeholder={t.languageSearch}
              />
            </Field>
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

function hostOk(host: string): boolean {
  if (!host || host.includes("@")) return false;
  if (host.startsWith("[")) return host.includes("]");
  const name = host.split(":")[0] ?? "";
  return name.toLowerCase() === "localhost" || name.includes(".");
}

function parseFeeds(text: string): { pack: string; url: string }[] | null {
  const feeds: { pack: string; url: string }[] = [];
  for (const line of text.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed) continue;
    const eq = trimmed.indexOf("=");
    if (eq < 2) return null;
    const pack = trimmed.slice(0, eq).trim();
    const url = trimmed.slice(eq + 1).trim();
    if (!/^[A-Za-z][A-Za-z0-9-]{1,15}$/.test(pack)) return null;
    const host = url.split("://")[1]?.split(/[/?#]/)[0] ?? "";
    if (!/^https?:\/\//.test(url) || !hostOk(host) || host.includes("@") || /\s/.test(url) || url.length > 200) return null;
    feeds.push({ pack, url });
    if (feeds.length > 8) return null;
  }
  return feeds;
}

function NewsFeedsField({
  t,
  settings,
  onSettings,
}: {
  t: Messages;
  settings: Settings;
  onSettings: (next: Settings) => void;
}) {
  const saved = (settings.news_feeds || []).map((feed) => `${feed.pack}=${feed.url}`).join("\n");
  const [draft, setDraft] = useState<string | null>(null);
  const text = draft ?? saved;
  const parsed = parseFeeds(text);
  const invalid = draft !== null && text.trim() !== "" && parsed === null;
  return (
    <Field>
      <FieldLabel htmlFor="klar-news-feeds">{t.newsFeeds}</FieldLabel>
      <Textarea
        id="klar-news-feeds"
        value={text}
        placeholder="de=https://example.com/feed.xml"
        aria-invalid={invalid || undefined}
        onChange={(ev) => {
          const next = ev.target.value;
          setDraft(next);
          const feeds = parseFeeds(next);
          if (feeds) onSettings({ ...settings, news_feeds: feeds });
        }}
        onBlur={() => {
          if (parsed) setDraft(null);
        }}
      />
      <FieldDescription>{invalid ? t.newsFeedsInvalid : t.newsFeedsHint}</FieldDescription>
    </Field>
  );
}

export function SettingsEngineSection({
  t,
  settings,
  loaded,
  onSettings,
  token,
  onToken,
  onApplyToken,
  theme,
  onTheme,
}: {
  t: Messages;
  settings: Settings;
  loaded: boolean;
  onSettings: (s: Settings) => void;
  token: string;
  onToken: (value: string) => void;
  onApplyToken: () => void;
  theme: Theme;
  onTheme: (theme: Theme) => void;
}) {
  return (
    <div className="grid min-w-0 gap-4">
      {loaded ? <Card>
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
            <NewsFeedsField t={t} settings={settings} onSettings={onSettings} />
            <SettingsToggle
              id="klar-allow-llm-tools"
              label={t.allowLlmTools}
              description={t.allowLlmToolsHint}
              checked={Boolean(settings.allow_llm_tools)}
              onCheckedChange={(checked) => onSettings({ ...settings, allow_llm_tools: checked })}
            />
          </FieldGroup>
        </CardContent>
      </Card> : null}
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
              <div className="flex gap-2">
                <Input id="klar-token" type="password" value={token} onChange={(ev) => onToken(ev.target.value)} />
                <Button type="button" variant="outline" disabled={!token.trim()} onClick={onApplyToken}>{t.apply}</Button>
              </div>
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
            <FieldDescription>
              {bundle
                ? fill(plural(bundle.count, { one: t.recordingsOne, other: t.recordingsOther }), {
                  count: String(bundle.count),
                })
                : "..."}
            </FieldDescription>
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
