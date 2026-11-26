import { useCallback, useEffect, useState } from "react";
import { api, type EngineHealth } from "../api";
import { fill, type Messages } from "../i18n";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

function homeAge(t: Messages, health: EngineHealth): string {
  const age = health.home.age_secs;
  if (age == null) return t.doctorAgeNever;
  return fill(t.doctorAge, { n: String(age) });
}

export function DoctorCard({ t }: { t: Messages }) {
  const [health, setHealth] = useState<EngineHealth | null>(null);
  const [error, setError] = useState(false);
  const load = useCallback(() => {
    setError(false);
    api
      .health()
      .then(setHealth)
      .catch(() => {
        setHealth(null);
        setError(true);
      });
  }, []);
  useEffect(load, [load]);
  return (
    <Card>
      <CardHeader>
        <CardTitle>{t.doctorTitle}</CardTitle>
        <CardDescription>{t.doctorHint}</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-4">
        {error || !health ? (
          <p className="caption text-destructive">{t.doctorUnavailable}</p>
        ) : (
          <>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorBuild}</h3>
              <p className="font-mono text-sm">
                {health.version}
                <span className="text-muted-foreground"> · </span>
                {health.sha}
              </p>
              <Badge variant={health.ok ? "default" : "destructive"}>{health.ok ? t.engineReady : t.engineOffline}</Badge>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorHome}</h3>
              <p className="text-sm">
                {health.home.source}
                {" · "}
                {fill(t.doctorEntities, { n: String(health.home.entities) })}
                {" · "}
                {homeAge(t, health)}
              </p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorOverlay}</h3>
              <p className="text-sm">{health.overlay.status}</p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorLlm}</h3>
              <p className="text-sm">
                {health.llm.configured ? t.llmConfigured : t.llmNotConfigured}
                {health.llm.provider ? ` · ${health.llm.provider}` : ""}
              </p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorWyoming}</h3>
              <p className="text-sm">{health.wyoming.listening ? t.engineReady : t.engineOffline}</p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{t.doctorOps}</h3>
              <p className="font-mono text-sm">
                req {health.ops.requests ?? 0}
                {" · "}
                overlay {health.ops.overlay_writes ?? 0}
                {" · "}
                wyoming {health.ops.wyoming_connects ?? 0}
              </p>
            </section>
          </>
        )}
      </CardContent>
    </Card>
  );
}
