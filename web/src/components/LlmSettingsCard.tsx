import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { LlmPublic } from "../types";
import { isProviderId, resolveProvider, writeStoredProvider } from "../llmProviders";
import { LlmProviderFields } from "./LlmProviderFields";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";

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
    if (isProviderId(next.provider)) writeStoredProvider(next.provider);
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
        provider: resolveProvider(baseUrl),
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
        <LlmProviderFields
          t={t}
          baseUrl={baseUrl}
          model={model}
          apiKey={apiKey}
          thinking={enableThinking}
          onUrl={setBaseUrl}
          onModel={setModel}
          onKey={setApiKey}
          onThinking={setEnableThinking}
        />
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        <Button type="button" onClick={() => void save()}>{t.save}</Button>
        <Button type="button" variant="ghost" disabled={!endpoint.configured} onClick={() => void clear()}>{t.llmClear}</Button>
        {status ? <p className="text-sm text-muted-foreground">{status}</p> : null}
      </CardFooter>
    </Card>
  );
}
