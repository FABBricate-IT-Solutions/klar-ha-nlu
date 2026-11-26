import { useEffect, useState } from "react";
import { api, type EngineHealth } from "../api";
import { doctorCopy, fillDoctor } from "../doctorCopy";
import type { Messages } from "../i18n";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

function homeAge(locale: string, health: EngineHealth): string {
  const copy = doctorCopy(locale);
  const age = health.home.age_secs;
  if (age == null) return copy.ageNever;
  return fillDoctor(copy.age, { n: String(age) });
}

export function DoctorCard({ t, locale }: { t: Messages; locale: string }) {
  const copy = doctorCopy(locale);
  const [health, setHealth] = useState<EngineHealth | null>(null);
  const [error, setError] = useState(false);

  useEffect(() => {
    let live = true;
    api
      .health()
      .then((next) => {
        if (!live) return;
        setHealth(next);
        setError(false);
      })
      .catch(() => {
        if (!live) return;
        setHealth(null);
        setError(true);
      });
    return () => {
      live = false;
    };
  }, []);

  return (
    <Card>
      <CardHeader>
        <CardTitle>{copy.title}</CardTitle>
        <CardDescription>{copy.hint}</CardDescription>
      </CardHeader>
      <CardContent className="grid gap-4">
        {error || !health ? (
          <p className="caption text-destructive">{copy.unavailable}</p>
        ) : (
          <>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.build}</h3>
              <p className="font-mono text-sm">
                {health.version}
                <span className="text-muted-foreground"> · </span>
                {health.sha}
              </p>
              <Badge variant={health.ok ? "default" : "destructive"}>{health.ok ? t.engineReady : t.engineOffline}</Badge>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.home}</h3>
              <p className="text-sm">
                {health.home.source}
                {" · "}
                {fillDoctor(copy.entities, { n: String(health.home.entities) })}
                {" · "}
                {homeAge(locale, health)}
              </p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.overlay}</h3>
              <p className="text-sm">{health.overlay.status}</p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.llm}</h3>
              <p className="text-sm">
                {health.llm.configured ? t.llmConfigured : t.llmNotConfigured}
                {health.llm.provider ? ` · ${health.llm.provider}` : ""}
              </p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.wyoming}</h3>
              <p className="text-sm">{health.wyoming.listening ? t.engineReady : t.engineOffline}</p>
            </section>
            <section className="grid gap-1">
              <h3 className="text-sm font-medium">{copy.ops}</h3>
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
