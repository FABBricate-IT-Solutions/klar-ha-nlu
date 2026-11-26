import { useEffect, useState } from "react";
import { toast } from "sonner";
import { api } from "../api";
import type { Messages } from "../i18n";
import { toastSaved, toastSaveFailed } from "../saveToast";
import { useLlmForm } from "../useLlmForm";
import { LlmProviderFields } from "./LlmProviderFields";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";

export function LlmSettingsCard({ t }: { t: Messages }) {
  const form = useLlmForm();
  const { load: fill, saved: endpoint } = form;
  const [busy, setBusy] = useState(false);
  // False until the stored endpoint was read; the empty fields are placeholders until then.
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    api.llmEndpoint().then((next) => {
      setLoaded(true);
      fill(next);
    }).catch((err) => {
      const text = err instanceof Error ? err.message : "";
      toast.error(text.startsWith("401") ? t.saveUnauthorized : t.trainerFail);
    });
  }, [fill, t.saveUnauthorized, t.trainerFail]);

  const save = async () => {
    if (!loaded || form.keyMissing) return;
    setBusy(true);
    try {
      const next = await api.saveLlmEndpoint(form.payload());
      fill(next);
      toastSaved(t, [next.model, next.base_url].filter(Boolean).join(" · "));
    } catch (err) {
      toastSaveFailed(t, err);
    } finally {
      setBusy(false);
    }
  };

  const clear = async () => {
    setBusy(true);
    try {
      fill(await api.saveLlmEndpoint({ configured: false }));
      toastSaved(t, t.llmNotConfigured);
    } catch (err) {
      toastSaveFailed(t, err);
    } finally {
      setBusy(false);
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
          <p className="font-mono text-xs text-muted-foreground">
            {endpoint.model} · {endpoint.base_url}
            {endpoint.source ? ` · ${t.source}: ${endpoint.source}` : null}
          </p>
        ) : null}
      </CardHeader>
      <CardContent>
        <LlmProviderFields t={t} form={form} />
      </CardContent>
      <CardFooter className="flex flex-wrap gap-2">
        <Button type="button" disabled={busy || !loaded || form.keyMissing} onClick={() => void save()}>{t.save}</Button>
        <Button type="button" variant="ghost" disabled={busy || !endpoint.configured} onClick={() => void clear()}>{t.llmClear}</Button>
      </CardFooter>
    </Card>
  );
}
