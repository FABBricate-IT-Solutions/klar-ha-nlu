import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { LlmPublic } from "../types";
import { LlmModelField } from "./LlmModelField";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

const OPENAI = "https://api.openai.com/v1";
const OLLAMA = "http://127.0.0.1:11434/v1";

function presetOf(url: string): "openai" | "ollama" | "" {
  if (url === OPENAI) return "openai";
  if (url === OLLAMA) return "ollama";
  return "";
}

export function LlmSettingsCard({ t }: { t: Messages }) {
  const [endpoint, setEndpoint] = useState<LlmPublic>({ configured: false });
  const [baseUrl, setBaseUrl] = useState("");
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [enableThinking, setEnableThinking] = useState(false);
  const [status, setStatus] = useState("");

  const load = async () => {
    const next = await api.llmEndpoint();
    setEndpoint(next);
    setBaseUrl(next.base_url || "");
    setModel(next.model || "");
    setApiKey("");
    setEnableThinking(Boolean(next.enable_thinking));
  };

  useEffect(() => {
    load().catch(() => setStatus(t.trainerFail));
  }, [t.trainerFail]);

  const save = async () => {
    try {
      const next = await api.saveLlmEndpoint({
        base_url: baseUrl,
        model,
        ...(apiKey.trim() ? { api_key: apiKey.trim() } : {}),
        configured: true,
        enable_thinking: enableThinking,
      });
      setEndpoint(next);
      setApiKey("");
      setStatus(t.save);
    } catch {
      setStatus(t.trainerFail);
    }
  };

  const clear = async () => {
    try {
      const next = await api.saveLlmEndpoint({ configured: false });
      setEndpoint(next);
      setBaseUrl("");
      setModel("");
      setApiKey("");
      setEnableThinking(false);
      setStatus(t.llmClear);
    } catch {
      setStatus(t.trainerFail);
    }
  };

  const preset = presetOf(baseUrl);

  return (
    <Card>
      <CardHeader>
        <div className="flex flex-wrap items-center gap-2">
          <CardTitle>{t.llm}</CardTitle>
          <Badge variant={endpoint.configured ? "default" : "outline"}>
            {endpoint.configured ? t.llmConfigured : t.llmNotConfigured}
          </Badge>
        </div>
        <CardDescription>{t.llmHint}</CardDescription>
        {endpoint.configured ? (
          <p className="font-mono text-xs text-muted-foreground">{endpoint.model} · {endpoint.base_url}</p>
        ) : null}
      </CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <ToggleGroup
              variant="outline"
              spacing={0}
              value={preset ? [preset] : []}
              onValueChange={(next) => {
                const picked = next[0];
                if (picked === "openai") setBaseUrl(OPENAI);
                if (picked === "ollama") setBaseUrl(OLLAMA);
              }}
            >
              <ToggleGroupItem value="openai">{t.llmPresetOpenAi}</ToggleGroupItem>
              <ToggleGroupItem value="ollama">{t.llmPresetOllama}</ToggleGroupItem>
            </ToggleGroup>
          </Field>
          <Field>
            <FieldLabel htmlFor="llm-base-url">{t.llmBaseUrl}</FieldLabel>
            <Input id="llm-base-url" value={baseUrl} onChange={(ev) => setBaseUrl(ev.target.value)} placeholder={OPENAI} />
          </Field>
          <Field>
            <FieldLabel htmlFor="llm-model">{t.llmModel}</FieldLabel>
            <LlmModelField t={t} baseUrl={baseUrl} apiKey={apiKey} model={model} onModel={setModel} />
          </Field>
          <Field>
            <FieldLabel htmlFor="llm-key">{t.llmApiKey}</FieldLabel>
            <Input id="llm-key" type="password" value={apiKey} onChange={(ev) => setApiKey(ev.target.value)} placeholder={t.llmApiKeyHint} autoComplete="off" />
            <FieldDescription>{t.llmApiKeyHint}</FieldDescription>
          </Field>
          <Field orientation="horizontal">
            <FieldContent>
              <FieldLabel>{t.llmThinking}</FieldLabel>
              <FieldDescription>{t.llmThinkingHint}</FieldDescription>
            </FieldContent>
            <Switch checked={enableThinking} onCheckedChange={(checked) => setEnableThinking(Boolean(checked))} />
          </Field>
        </FieldGroup>
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        <Button type="button" onClick={() => void save()}>{t.save}</Button>
        <Button type="button" variant="ghost" disabled={!endpoint.configured} onClick={() => void clear()}>{t.llmClear}</Button>
        {status ? <p className="text-sm text-muted-foreground">{status}</p> : null}
      </CardFooter>
    </Card>
  );
}
