import { useEffect, useState } from "react";
import { Loader2Icon } from "lucide-react";
import { api, type LangOverlay } from "../api";
import type { PolicyLane } from "./PolicyPath";
import type { Messages } from "../i18n";
import type {
  LlmPublic,
  MatchControl,
  PolicyRule,
  TrainerChatEvent,
  TrainerProposal,
  TrainerTurn,
  TrainerValidateOut,
} from "../types";
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

function layerOf(proposal: TrainerProposal): "match" | "language" | "house" | "all" {
  const layer = proposal.layer || "all";
  switch (layer) {
    case "match":
    case "language":
    case "house":
    case "all":
      return layer;
    default:
      return "all";
  }
}

function applyEvent(
  event: TrainerChatEvent,
  setLines: (fn: (prev: TrainerTurn[]) => TrainerTurn[]) => void,
  setProposal: (next: TrainerProposal | null) => void,
  setRaw: (next: string) => void,
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
    case "proposal":
      setProposal(event.value);
      setRaw(JSON.stringify(event.value, null, 2));
      return;
    case "validate":
      setResult(event.value);
      onStatus(event.value.ok ? t.trainerOk : t.trainerFail);
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
  overlay,
  onApplyHouse,
  onApplyMatch,
  onStatus,
}: {
  t: Messages;
  lane: PolicyLane;
  language?: string;
  overlay: LangOverlay | null;
  onApplyHouse: (next: PolicyRule[]) => Promise<void>;
  onApplyMatch: (next: MatchControl[]) => Promise<void>;
  onStatus: (status: string) => void;
}) {
  const [endpoint, setEndpoint] = useState<LlmPublic | null>(null);
  const [draft, setDraft] = useState("");
  const [lines, setLines] = useState<TrainerTurn[]>([]);
  const [busy, setBusy] = useState(false);
  const [raw, setRaw] = useState("");
  const [result, setResult] = useState<TrainerValidateOut | null>(null);
  const [proposal, setProposal] = useState<TrainerProposal | null>(null);

  useEffect(() => {
    api.llmEndpoint().then(setEndpoint).catch(() => setEndpoint({ configured: false }));
  }, []);

  const parsed = (): TrainerProposal | null => {
    if (proposal) return proposal;
    if (!raw.trim()) return null;
    try {
      return JSON.parse(raw) as TrainerProposal;
    } catch {
      return null;
    }
  };

  const send = async () => {
    const message = draft.trim();
    if (!message || busy) return;
    setDraft("");
    const history = lines.slice(-8);
    setLines((prev) => [...prev, { role: "user", content: message }, { role: "assistant", content: "" }]);
    setBusy(true);
    setResult(null);
    setProposal(null);
    try {
      await api.trainerChat({ message, layer: trainerLayer(lane), language, history }, (event) => {
        applyEvent(event, setLines, setProposal, setRaw, setResult, onStatus, t);
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

  const runValidate = async () => {
    const next = parsed();
    if (!next) {
      onStatus(t.trainerFail);
      return;
    }
    if (!next.language && language) next.language = language;
    const out = await api.validateProposal(next);
    setResult(out);
    onStatus(out.ok ? t.trainerOk : t.trainerFail);
  };

  const applyLane = async (lane: "house" | "match" | "language") => {
    if (!result?.ok) return;
    const next = parsed();
    if (!next) return;
    const layer = layerOf(next);
    if (layer !== "all" && layer !== lane) return;
    if (lane === "house" && next.policies) await onApplyHouse(next.policies);
    if (lane === "match" && next.match_controls) await onApplyMatch(next.match_controls);
    if (lane === "language" && next.language_overlay) {
      await api.saveLangOverlay({ custom: overlay?.custom || [], language: next.language_overlay, label: "trainer" });
    }
    onStatus(t.trainerApply);
  };

  const canApply = (lane: "house" | "match" | "language") => {
    if (!result?.ok) return false;
    const next = parsed();
    if (!next) return false;
    const layer = layerOf(next);
    if (layer !== "all" && layer !== lane) return false;
    if (lane === "house") return Boolean(next.policies);
    if (lane === "match") return Boolean(next.match_controls);
    return Boolean(next.language_overlay);
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
              </div>
            </ScrollArea>
            <FieldGroup>
              <Field>
                <FieldLabel htmlFor="trainer-draft">{t.trainerSend}</FieldLabel>
                <Textarea
                  id="trainer-draft"
                  className="min-h-24 font-mono text-sm"
                  value={draft}
                  disabled={busy}
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
            <Button type="button" disabled={busy || !draft.trim()} onClick={() => void send()}>
              {busy ? <Loader2Icon data-icon="inline-start" className="animate-spin" /> : null}
              {busy ? t.trainerStreaming : t.trainerSend}
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
            <div className="flex flex-wrap gap-2">
              <Button type="button" disabled={!canApply("house")} onClick={() => void applyLane("house")}>{t.trainerApplyHouse}</Button>
              <Button type="button" disabled={!canApply("match")} onClick={() => void applyLane("match")}>{t.trainerApplyMatch}</Button>
              <Button type="button" disabled={!canApply("language")} onClick={() => void applyLane("language")}>{t.trainerApplyLanguage}</Button>
            </div>
            <Collapsible>
              <CollapsibleTrigger render={<Button variant="outline" type="button" />}>
                {t.trainerAdvanced}
              </CollapsibleTrigger>
              <CollapsibleContent className="flex flex-col gap-3 pt-3">
                <Button variant="secondary" type="button" onClick={() => void runValidate()}>{t.trainerValidate}</Button>
                <Field>
                  <FieldLabel htmlFor="trainer-proposal">{t.trainerProposal}</FieldLabel>
                  <Textarea
                    id="trainer-proposal"
                    className="min-h-40 font-mono text-xs"
                    value={raw}
                    onChange={(ev) => { setRaw(ev.target.value); setProposal(null); }}
                  />
                </Field>
              </CollapsibleContent>
            </Collapsible>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
