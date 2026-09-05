import { useEffect, useState } from "react";
import { api, download, setToken, type LanguagePack } from "../api";
import { LlmSettingsCard } from "../components/LlmSettingsCard";
import { SearchSelect, withCurrent } from "../components/SearchSelect";
import { dictionaries, type Messages } from "../i18n";
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
  const de = document.documentElement.lang.startsWith("de");
  const [bundle, setBundle] = useState<BundleList | null>(null);
  const [confirmClear, setConfirmClear] = useState(false);
  const [token, setTokenValue] = useState(localStorage.getItem("klar_token") || "");
  const [picked, setPicked] = useState<Theme>(() => readTheme(theme));
  const [packs, setPacks] = useState<LanguagePack[]>([]);
  const refresh = () => api.bundle().then(setBundle).catch(() => undefined);
  useEffect(() => {
    refresh();
  }, []);
  useEffect(() => {
    api.languages().then(setPacks).catch(() => undefined);
  }, []);
  const chromeCodes = new Set(Object.keys(dictionaries));
  const localeOptions = (packs.length ? packs.filter((pack) => chromeCodes.has(pack.code)) : [...chromeCodes].map((code) => ({
    code,
    native_name: code,
    script: "",
    variants: [code],
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
    onTheme?.(next);
  };
  return (
    <div className="page flex flex-col gap-6">
      <section className="hero">
        <div>
          <h1>{t.settings}</h1>
          <p className="muted">{t.engineHint}</p>
        </div>
        <div className="flex flex-wrap gap-2">
          <Button variant="ghost" type="button" onClick={() => onReplayWizard?.()}>
            {de ? "Setup erneut" : "Replay setup"}
          </Button>
          <Button type="button" onClick={() => void save()}>{t.save}</Button>
        </div>
      </section>
      <LlmSettingsCard t={t} />
      <section className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>{t.personalityHa}</CardTitle>
            <CardDescription>{de ? "Darstellung und Operator-Sprache" : "Appearance and operator language"}</CardDescription>
          </CardHeader>
          <CardContent>
            <FieldGroup>
              <Field>
                <FieldLabel>{de ? "Darstellung" : "Appearance"}</FieldLabel>
                <ToggleGroup
                  variant="outline"
                  spacing={0}
                  value={[picked]}
                  onValueChange={(next) => {
                    const value = next[0];
                    if (value === "dark" || value === "light") setTheme(value);
                  }}
                  aria-label={de ? "Darstellung" : "Appearance"}
                >
                  <ToggleGroupItem value="dark">{de ? "Dunkel" : "Dark"}</ToggleGroupItem>
                  <ToggleGroupItem value="light">{de ? "Hell" : "Light"}</ToggleGroupItem>
                </ToggleGroup>
              </Field>
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
                <FieldLabel>{t.languages}</FieldLabel>
                <FieldDescription>{t.languageHint}</FieldDescription>
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
              <Field>
                <FieldLabel htmlFor="klar-token">{t.token}</FieldLabel>
                <Input id="klar-token" type="password" value={token} onChange={(ev) => setTokenValue(ev.target.value)} />
              </Field>
            </FieldGroup>
          </CardContent>
        </Card>
        <Card>
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
