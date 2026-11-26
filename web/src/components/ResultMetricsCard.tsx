import { Bars } from "./charts";
import { Empty } from "./common";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { download } from "../api";
import { fill, type Messages } from "../i18n";
import type { ResultWindow } from "../types";

export function ResultMetricsCard({
  results,
  t,
}: {
  results?: ResultWindow | null;
  t: Messages;
}) {
  const top = results?.top_rejections ?? [];
  const mix = results?.decision_mix;
  const matchers = results?.matcher_hit_rates ?? [];
  const repeats = results?.repeats_within_30s ?? 0;
  const hasData = (results?.total ?? 0) > 0;

  return (
    <Card className="mb-4 overflow-visible">
      <CardHeader className="flex flex-col items-stretch justify-between gap-3 sm:flex-row sm:items-start">
        <div>
          <CardTitle>{t.resultMetrics}</CardTitle>
          <CardDescription>{t.resultMetricsCaption}</CardDescription>
        </div>
        <Button
          variant="outline"
          type="button"
          onClick={() => void download("/api/v2/result-metrics/export", "klar-result-metrics.json")}
        >
          {t.resultMetricsExport}
        </Button>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        {!hasData ? <Empty text={t.resultMetricsEmpty} /> : null}
        {hasData && mix ? (
          <div className="grid gap-4 md:grid-cols-2">
            <div>
              <h3 className="mb-2 text-sm font-medium">{t.decisionMix}</h3>
              <Bars
                data={[
                  { label: "execute", value: mix.execute },
                  { label: "confirm", value: mix.confirm },
                  { label: "clarify", value: mix.clarify },
                  { label: "reject", value: mix.reject },
                  { label: "chat", value: mix.chat },
                  { label: "error", value: mix.error },
                ].filter((row) => row.value > 0)}
                unit={t.unitsTurns}
              />
            </div>
            <div>
              <h3 className="mb-2 text-sm font-medium">{t.topRejections}</h3>
              <p className="caption mb-2">{t.topRejectionsCaption}</p>
              {top.length === 0 ? (
                <Empty text={t.topRejectionsEmpty} />
              ) : (
                <ol className="flex flex-col gap-1" aria-label={t.topRejections}>
                  {top.map((row) => (
                    <li
                      className="flex items-center justify-between gap-3 border-b border-border py-1.5 text-sm"
                      key={row.reason}
                      data-testid={`top-rejection-${row.reason}`}
                    >
                      <span>{row.reason}</span>
                      <span className="muted">{fill(t.rejectionCount, { count: String(row.count) })}</span>
                    </li>
                  ))}
                </ol>
              )}
            </div>
          </div>
        ) : null}
        {hasData ? (
          <p className="text-sm" data-testid="repeats-within-30s">
            {fill(t.repeatsWithin30s, { count: String(repeats) })}
          </p>
        ) : null}
        {hasData && matchers.length > 0 ? (
          <div>
            <h3 className="mb-2 text-sm font-medium">{t.matcherHitRates}</h3>
            <Bars
              data={matchers.map((row) => ({
                label: row.id,
                value: Math.round(row.rate * 100),
              }))}
              unit={t.unitsPercent}
            />
          </div>
        ) : null}
      </CardContent>
    </Card>
  );
}
