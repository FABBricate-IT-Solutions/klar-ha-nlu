import { useId } from "react";
import { Area, AreaChart, Bar, BarChart, CartesianGrid, Cell, Pie, PieChart, XAxis, YAxis } from "recharts";
import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from "@/components/ui/chart";

type Slice = { label: string; value: number; fill: string };

export function Donut({ high, medium, low }: { high: number; medium: number; low: number }) {
  const data: Slice[] = [
    { label: "high", value: high, fill: "var(--chart-1)" },
    { label: "medium", value: medium, fill: "var(--chart-3)" },
    { label: "low", value: low, fill: "var(--chart-4)" },
  ].filter((item) => item.value > 0);
  const total = high + medium + low || 1;
  const config: ChartConfig = {
    high: { label: "high", color: "var(--chart-1)" },
    medium: { label: "medium", color: "var(--chart-3)" },
    low: { label: "low", color: "var(--chart-4)" },
  };
  return (
    <div className="relative">
      <ChartContainer config={config} className="mx-auto aspect-square max-h-[220px]" aria-label="confidence">
        <PieChart>
          <ChartTooltip content={<ChartTooltipContent hideLabel />} />
          <Pie data={data} dataKey="value" nameKey="label" innerRadius={58} outerRadius={88} strokeWidth={2} paddingAngle={2}>
            {data.map((item) => (
              <Cell key={item.label} fill={item.fill} />
            ))}
          </Pie>
        </PieChart>
      </ChartContainer>
      <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
        <span className="text-3xl font-semibold tracking-tight">{Math.round((high / total) * 100)}%</span>
        <span className="text-xs text-muted-foreground">ready</span>
      </div>
    </div>
  );
}

export function Bars({ data, unit = "" }: { data: { label: string; value: number }[]; unit?: string }) {
  const config: ChartConfig = {
    value: { label: unit || "value", color: "var(--chart-2)" },
  };
  const rows = data.length ? data : [{ label: "—", value: 0 }];
  return (
    <ChartContainer config={config} className="aspect-auto h-[220px] w-full">
      <BarChart data={rows} layout="vertical" accessibilityLayer margin={{ left: 8, right: 12 }}>
        <CartesianGrid horizontal={false} strokeDasharray="3 3" />
        <XAxis type="number" hide />
        <YAxis type="category" dataKey="label" width={108} tickLine={false} axisLine={false} />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Bar dataKey="value" fill="var(--color-value)" radius={4} />
      </BarChart>
    </ChartContainer>
  );
}

export function AreaTrend({ data, unit = "turns" }: { data: { day: string; count: number }[]; unit?: string }) {
  const gradientId = useId().replace(/:/g, "");
  const points = data.length ? data : [{ day: "—", count: 0 }];
  const config: ChartConfig = {
    count: { label: unit, color: "var(--chart-2)" },
  };
  return (
    <ChartContainer config={config} className="aspect-auto h-[200px] w-full">
      <AreaChart data={points} accessibilityLayer margin={{ left: 8, right: 8 }}>
        <defs>
          <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
            <stop offset="5%" stopColor="var(--color-count)" stopOpacity={0.35} />
            <stop offset="95%" stopColor="var(--color-count)" stopOpacity={0.04} />
          </linearGradient>
        </defs>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis dataKey="day" tickLine={false} axisLine={false} tickFormatter={(value) => String(value).slice(-5)} />
        <YAxis tickLine={false} axisLine={false} width={28} />
        <ChartTooltip content={<ChartTooltipContent />} />
        <Area dataKey="count" type="natural" fill={`url(#${gradientId})`} stroke="var(--color-count)" strokeWidth={1.75} />
      </AreaChart>
    </ChartContainer>
  );
}

export type MixRow = { day: string; execute: number; confirm: number; clarify: number; reject: number; chat: number };

const MIX_KEYS = ["execute", "confirm", "clarify", "reject", "chat"] as const;

const MIX_CONFIG: ChartConfig = {
  execute: { label: "execute", color: "var(--chart-1)" },
  confirm: { label: "confirm", color: "var(--chart-2)" },
  clarify: { label: "clarify", color: "var(--chart-3)" },
  reject: { label: "reject", color: "var(--chart-4)" },
  chat: { label: "chat", color: "var(--chart-5)" },
};

export function DecisionMix({ data, unit }: { data: MixRow[]; unit: string }) {
  const rows = data.length ? data : [{ day: "—", execute: 0, confirm: 0, clarify: 0, reject: 0, chat: 0 }];
  return (
    <ChartContainer config={MIX_CONFIG} className="aspect-auto h-[220px] w-full" aria-label="decision mix">
      <BarChart data={rows} accessibilityLayer margin={{ left: 8, right: 8 }}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis dataKey="day" tickLine={false} axisLine={false} tickFormatter={(value) => String(value).slice(-5)} />
        <YAxis tickLine={false} axisLine={false} width={28} />
        <ChartTooltip content={<ChartTooltipContent />} />
        {MIX_KEYS.map((key, index) => (
          <Bar
            key={key}
            dataKey={key}
            stackId="mix"
            fill={`var(--color-${key})`}
            radius={index === MIX_KEYS.length - 1 ? 4 : 0}
          />
        ))}
      </BarChart>
    </ChartContainer>
  );
}

export function StageBars({ data, unit }: { data: { label: string; value: number }[]; unit: string }) {
  return <Bars data={data} unit={unit} />;
}
