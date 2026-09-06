import { useEffect, useState } from "react";
import { Loader2Icon } from "lucide-react";
import { api } from "../api";
import type { PolicyLane } from "./PolicyPath";
import type { Messages } from "../i18n";
import type { LlmPublic, TrainerChatEvent, TrainerConsent, TrainerTurn, TrainerValidateOut } from "../types";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "cn";

function trainerLayer(lane: PolicyLane): "match" | "language" | "house" {
  switch (lane) {
    case "match":
    case "language":
    case "house":
      return lane;
    default: {
      const _never: never = lane;
      return _never;
    }
  }
}

function applyEvent(
  event: TrainerChatEvent,
  setLines: (fn: (prev: TrainerTurn[]) => TrainerTurn[]) => void,
  setConsent: (next: TrainerConsent | null) => void,
  setYolo: (next: boolean) => void,
  setResult: (next: TrainerValidateOut | null) => void,
  onStatus: (status: string) => void,
  t: Messages,
) {
  switch (event.type) {
    case "delta":
      setLines((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role === "assistant") {
          next[next.length - 1] = { role: "assistant", content: last.content + event.text };
        }
        return next;
      });
      return;
    case "consent":
      setConsent({
        call_id: event.call_id,
        tool: event.tool,
        summary: event.summary,
        validate: event.validate,
      });
      setResult(event.validate);
      return;
    case "session":
      setYolo(event.yolo);
      return;
    case "validate":
      setResult(event.value);
      onStatus(event.value.ok ? t.trainerOk : t.trainerFail);
      return;
    case "proposal":
      return;
    case "tool_call":
      return;
    case "done":
      setLines((prev) => {
        const next = [...prev];
        const last = next[next.length - 1];
        if (last?.role === "assistant" && !last.content.trim() && event.text) {
          next[next.length - 1] = { role: "assistant", content: event.text };
        }
        return next;
      });
      return;
    case "error":
      onStatus(event.message || t.trainerFail);
      return;
    default: {
      const _never: never = event;
      return _never;
    }
  }
}

export function TrainerDrawer({
  t,
  lane,
  language,
  onStatus,
}: {
  t: Messages;
  lane: PolicyLane;
  language?: string;
  overlay?: unknown;
  onApplyHouse?: unknown;
  onApplyMatch?: unknown;
  onStatus: (status: string) => void;
}) {
  const [endpoint, setEndpoint] = useState<LlmPublic | null>(null);
  const [draft, setDraft] = useState("");
  const [lines, setLines] = useState<TrainerTurn[]>([]);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<TrainerValidateOut | null>(null);
  const [consent, setConsent] = useState<TrainerConsent | null>(null);
  const [yolo, setYolo] = useState(false);

  useEffect(() => {
    api.llmEndpoint().then(setEndpoint).catch(() => setEndpoint({ configured: false }));
  }, []);

  const send = async () => {
    const message = draft.trim();
    if (!message || busy) return;
    setDraft("");
    const history = lines.slice(-8);
    setLines((prev) => [...prev, { role: "user", content: message }, { role: "assistant", content: "" }]);
    setBusy(true);
    setResult(null);
    setConsent(null);
    try {
      await api.trainerChat({ message, layer: trainerLayer(lane), language, history }, (event) => {
        applyEvent(event, setLines, setConsent, setYolo, setResult, onStatus, t);
      });
    } catch (err) {
      if (err instanceof Error && err.message === "llm-unconfigured") {
        setEndpoint({ configured: false });
        onStatus(t.trainerNeedLlm);
      } else {
        onStatus(t.trainerFail);
      }
    } finally {
      setBusy(false);
    }
  };

  const decide = async (decision: "allow_once" | "allow" | "yolo" | "deny" | "ask_again") => {
    try {
      const out = await api.trainerConsent({ call_id: consent?.call_id, decision });
      setYolo(out.yolo);
      if (decision !== "ask_again") {
        setConsent(null);
      }
    } catch {
      onStatus(t.trainerFail);
    }
  };

  if (!endpoint || !endpoint.configured) {
    return (
      <Card className="mt-4 ring-1 ring-primary/40">
        <CardHeader>
          <CardTitle>{t.trainerForLane}</CardTitle>
          <CardDescription>{endpoint ? t.trainerNeedLlm : t.trainerStreaming}</CardDescription>
        </CardHeader>
        {endpoint ? (
          <CardContent>
            <Button type="button" onClick={() => { window.location.hash = "#/settings"; }}>{t.trainerOpenSettings}</Button>
          </CardContent>
        ) : null}
      </Card>
    );
  }

  return (
    <Card className="mt-4">
      <CardHeader>
        <div className="flex flex-wrap items-center gap-2">
          <CardTitle>{t.trainerForLane}</CardTitle>
          <Badge variant="outline">{endpoint.model || "LLM"}</Badge>
          {yolo ? (
            <Badge variant="outline">
              {t.trainerYolo}
              <Button type="button" variant="ghost" size="sm" className="ml-2 h-6 px-2" onClick={() => void decide("ask_again")}>
                {t.trainerAskAgain}
              </Button>
            </Badge>
          ) : null}
        </div>
        <CardDescription>{t.trainerHint}</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid gap-4 md:grid-cols-2">
          <div className="flex flex-col gap-3">
            <ScrollArea className="h-80 rounded-lg border bg-background">
              <div className="flex flex-col gap-2 p-3">
                {lines.map((line, index) => (
                  <p
                    key={`${line.role}-${index}`}
                    className={cn(
                      "min-h-11 rounded-lg px-3 py-2 text-sm",
                      line.role === "user"
                        ? "self-end bg-primary/15 text-foreground"
                        : "self-start bg-muted text-foreground",
                    )}
                  >
                    {line.content || (busy ? t.trainerStreaming : "")}
                  </p>
                ))}
                {consent ? (
                  <div className="rounded-lg border bg-background p-3 text-sm">
                    <p className="font-medium">{consent.tool}</p>
                    <p className="text-muted-foreground">{consent.summary}</p>
                    <div className="mt-2 flex flex-wrap gap-2">
                      <Button type="button" size="sm" onClick={() => void decide("allow")}>{t.trainerAllow}</Button>
                      <Button type="button" size="sm" variant="secondary" onClick={() => void decide("allow_once")}>{t.trainerAllowOnce}</Button>
                      <Button type="button" size="sm" variant="secondary" onClick={() => void decide("yolo")}>{t.trainerYolo}</Button>
                      <Button type="button" size="sm" variant="outline" onClick={() => void decide("deny")}>{t.trainerDeny}</Button>
                    </div>
                  </div>
                ) : null}
              </div>
            </ScrollArea>
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="trainer-draft">{t.trainerSend}</FieldLabel>
                <Textarea
                  id="trainer-draft"
                  className="min-h-24 font-mono text-sm"
                  value={draft}
                  disabled={busy && !consent}
                  onChange={(ev) => setDraft(ev.target.value)}
                  onKeyDown={(ev) => {
                    if (ev.key === "Enter" && !ev.shiftKey) {
                      ev.preventDefault();
                      void send();
                    }
                  }}
                />
              </Field>
            </FieldGroup>
            <Button type="button" disabled={(busy && !consent) || !draft.trim()} onClick={() => void send()}>
              {busy && !consent ? <Loader2Icon data-icon="inline-start" className="animate-spin" /> : null}
              {busy && !consent ? t.trainerStreaming : t.trainerSend}
            </Button>
          </div>
          <div className="flex flex-col gap-3">
            {result ? (
              <Alert variant={result.ok ? "default" : "destructive"}>
                <AlertTitle>{result.ok ? t.trainerOk : t.trainerFail}</AlertTitle>
                <AlertDescription>
                  {result.errors.map((row) => (
                    <p key={`${row.path}-${row.message}`}>{row.path}: {row.message}</p>
                  ))}
                  {result.warnings.map((row) => (
                    <p key={`w-${row.path}-${row.message}`}>{row.path}: {row.message}</p>
                  ))}
                  {result.dry_run.map((row) => (
                    <p className="font-mono" key={row.text}>
                      {row.text} → {row.decision}
                      {row.seed ? ` · seed ${row.seed}` : ""}
                      {row.house ? ` · house ${row.house}` : ""}
                    </p>
                  ))}
                </AlertDescription>
              </Alert>
            ) : null}
            <Collapsible>
              <CollapsibleTrigger render={<Button variant="outline" type="button" />}>
                {t.trainerAdvanced}
              </CollapsibleTrigger>
              <CollapsibleContent className="pt-3 text-sm text-muted-foreground">
                {t.trainerHint}
              </CollapsibleContent>
            </Collapsible>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
