import { useEffect, useState } from "react";
import { api, download, setToken, type LanguagePack } from "../api";
import { Guide } from "../components/Guide";
import { CustomVoiceInterview } from "../components/CustomVoiceInterview";
import { LlmSettingsCard } from "../components/LlmSettingsCard";
import { SettingsBackupCard } from "../components/SettingsBackupCard";
import { PersonalityPrompt } from "../components/PersonalityPrompt";
import { SearchSelect, withCurrent } from "../components/SearchSelect";
import { dictionaries, type Messages } from "../i18n";
import { isPersonality, PERSONALITIES, personalityLabel } from "../personality";
import type { BundleList, Locale, Settings, Theme } from "../types";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

function readTheme(theme?: Theme): Theme {
  if (theme === "light" || theme === "dark") return theme;
  return document.documentElement.dataset.theme === "light" ? "light" : "dark";
}

export function SettingsPage({
  t,
  locale,
  onLocale,
  settings,
  onSettings,
  onReplayWizard,
  theme,
  onTheme,
}: {
  t: Messages;
  locale: Locale;
  onLocale: (locale: Locale) => void;
  settings: Settings;
  onSettings: (s: Settings) => void;
  onReplayWizard?: () => void;
  theme?: Theme;
  onTheme?: (theme: Theme) => void;
}) {
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const [picked, setPicked] = useState<Theme>(() => readTheme(theme));
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const [llmEpoch, setLlmEpoch] = useState(0);
  const [llmReady, setLlmReady] = useState(false);
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  useEffect(() => {
    api.languages().then(setPacks).catch(() => undefined);
    api.llmEndpoint().then((next) => setLlmReady(Boolean(next.configured))).catch(() => undefined);
  }, []);
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
  useEffect(() => {
    if (theme === "light" || theme === "dark") setPicked(theme);
  }, [theme]);
  const save = async (next = settings) => {
    setToken(token);
    onSettings(await api.saveSettings(next));
    refresh();
  };
  const clear = async () => {
    await api.clearBundle();
    refresh();
    setConfirmClear(false);
  };
  const setTheme = (next: Theme) => {
    setPicked(next);
    document.documentElement.dataset.theme = next;
    document.documentElement.classList.toggle("dark", next !== "light");
    try {
      localStorage.setItem("klar_theme", next);
    } catch {
      /* private mode */
    }
    onTheme?.(next);
  };
  const allAssist = settings.languages.length === 0;
  const pinned = settings.languages[0] || locale;
  const voice = isPersonality(settings.personality) ? settings.personality : "default";
  return (
    <div className="page flex flex-col gap-6">
      <section className="hero">
        <div>
          <h1>{t.settings}</h1>
          <p className="muted">{t.engineHint}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="ghost" type="button" onClick={() => onReplayWizard?.()}>
            {t.setupReplay}
          </Button>
          <Button type="button" onClick={() => void save()}>{t.save}</Button>
        </div>
      </section>
      <Guide
        title={t.settingsGuide}
        steps={[
          { id: "voice", label: t.settingsGuideVoice, hint: t.voiceHint },
          { id: "llm", label: t.settingsGuideLlm, hint: t.llmHint },
          { id: "lang", label: t.settingsGuideLang, hint: t.assistLanguagesHint },
        ]}
      />
      <p className="caption" style={{ marginTop: -8 }}>{t.haGlueHint}</p>
      <LlmSettingsCard key={llmEpoch} t={t} />
      <section className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t.voice}</CardTitle>
            <CardDescription>{t.voiceHint}</CardDescription>
          </CardHeader>
          <CardContent>
            <FieldGroup>
              <Field>
                <FieldLabel>{t.personality}</FieldLabel>
                <Select
                  value={voice}
                  onValueChange={(value) => {
                    if (value && isPersonality(value)) onSettings({ ...settings, personality: value });
                  }}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      {PERSONALITIES.map((id) => (
                        <SelectItem key={id} value={id}>{personalityLabel(t, id)}</SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Field>
              {voice === "custom" ? (
                <CustomVoiceInterview
                  t={t}
                  language={pinned}
                  value={settings.custom_voice || ""}
                  onChange={(prompt) => onSettings({ ...settings, personality: "custom", custom_voice: prompt })}
                />
              ) : (
                <PersonalityPrompt t={t} personality={voice} language={pinned} />
              )}
              {llmReady && voice !== "custom" ? (
                <Field>
                  <Button
                    variant="outline"
                    type="button"
                    onClick={() => onSettings({ ...settings, personality: "custom" })}
                  >
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
            <CardTitle>{t.missTitle}</CardTitle>
            <CardDescription>{t.missHint}</CardDescription>
          </CardHeader>
          <CardContent>
            <FieldGroup>
              <Field>
                <FieldLabel>{t.mode}</FieldLabel>
                <Select
                  value={settings.mode}
                  onValueChange={(value) => {
                    if (value === "full" || value === "context_only") {
                      onSettings({ ...settings, mode: value });
                    }
                  }}
                >
                  <SelectTrigger className="w-full">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectGroup>
                      <SelectItem value="full">{t.modeFull}</SelectItem>
                      <SelectItem value="context_only">{t.modeContext}</SelectItem>
                    </SelectGroup>
                  </SelectContent>
                </Select>
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
                <FieldLabel>{t.operatorLanguage}</FieldLabel>
                <SearchSelect
                  value={locale}
                  options={withCurrent(localeOptions, locale)}
                  onChange={onLocale}
                  allowEmpty={false}
                  placeholder={t.languageSearch}
                />
                <FieldDescription>{t.operatorLanguageHint}</FieldDescription>
              </Field>
              <Field>
                <FieldLabel>{t.operatorChrome}</FieldLabel>
                <ToggleGroup
                  variant="outline"
                  spacing={0}
                  value={[picked]}
                  onValueChange={(next) => {
                    const value = next[0];
                    if (value === "dark" || value === "light") setTheme(value);
                  }}
                  aria-label={t.operatorChrome}
                >
                  <ToggleGroupItem value="dark">{t.appearanceDark}</ToggleGroupItem>
                  <ToggleGroupItem value="light">{t.appearanceLight}</ToggleGroupItem>
                </ToggleGroup>
              </Field>
              <Field>
                <FieldLabel htmlFor="klar-token">{t.token}</FieldLabel>
                <Input id="klar-token" type="password" value={token} onChange={(ev) => setTokenValue(ev.target.value)} />
              </Field>
            </FieldGroup>
          </CardContent>
        </Card>
        <Card className="md:col-span-2">
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
                  onCheckedChange={(checked) => void save({ ...settings, support_bundle: Boolean(checked) })}
                />
              </Field>
              <Field orientation="horizontal">
                <FieldContent>
                  <FieldLabel>{t.includeRawText}</FieldLabel>
                </FieldContent>
                <Switch
                  checked={settings.support_bundle_raw_text}
                  onCheckedChange={(checked) => void save({ ...settings, support_bundle_raw_text: Boolean(checked) })}
                />
              </Field>
              <Field orientation="horizontal">
                <FieldContent>
                  <FieldLabel>{t.semanticAdapters}</FieldLabel>
                </FieldContent>
                <Switch
                  checked={settings.semantic_adapters}
                  onCheckedChange={(checked) => void save({ ...settings, semantic_adapters: Boolean(checked) })}
                />
              </Field>
              <Field>
                <FieldLabel>{t.journal}</FieldLabel>
                <FieldDescription>{bundle ? `${bundle.count} ${t.recordings}` : "..."}</FieldDescription>
              </Field>
            </FieldGroup>
          </CardContent>
          <CardFooter className="flex flex-wrap gap-2">
            <Button variant="outline" type="button" onClick={() => download("/api/bundle/dataset", "klar-assist-dataset.yaml")}>{t.downloadDataset}</Button>
            <Button variant="outline" type="button" onClick={() => download("/api/bundle/protocol", "klar-support-bundle.jsonl")}>{t.downloadProtocol}</Button>
            <Button variant="destructive" type="button" onClick={() => setConfirmClear(true)}>{t.clearAll}</Button>
          </CardFooter>
        </Card>
        <SettingsBackupCard
          t={t}
          onSettings={onSettings}
          onRestored={() => setLlmEpoch((epoch) => epoch + 1)}
        />
      </section>
      <AlertDialog open={confirmClear} onOpenChange={setConfirmClear}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t.clearAll}</AlertDialogTitle>
            <AlertDialogDescription>{t.journalHint}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void clear()}>{t.clearAll}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
