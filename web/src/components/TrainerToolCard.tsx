function asRecord(raw: string | undefined): Record<string, unknown> {
  if (!raw?.trim()) {
    return {};
  }
  try {
    const value = JSON.parse(raw) as unknown;
    return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, unknown>) : {};
  } catch {
    return {};
  }
}

function asRows(value: unknown): Record<string, unknown>[] {
  return Array.isArray(value) ? value.filter((row): row is Record<string, unknown> => Boolean(row) && typeof row === "object") : [];
}

function asStrings(value: unknown): string[] {
  return Array.isArray(value) ? value.map((row) => String(row)) : [];
}

function textOf(row: Record<string, unknown>, key: string): string {
  const value = row[key];
  return value == null ? "" : String(value);
}

function Rows({ rows, primary, secondary }: { rows: Record<string, unknown>[]; primary: string; secondary?: string }) {
  if (rows.length === 0) {
    return null;
  }
  return (
    <ul className="trainer-card-list">
      {rows.slice(0, 16).map((row, index) => {
        const title = textOf(row, primary);
        const extra = secondary ? textOf(row, secondary) : "";
        return (
          <li key={`${title}-${index}`}>
            <span>{title}</span>
            {extra ? <code>{extra}</code> : null}
          </li>
        );
      })}
    </ul>
  );
}

export function TrainerToolCard({
  name,
  args,
  result,
}: {
  name: string;
  args: string;
  result?: string;
}) {
  const parsedArgs = asRecord(args);
  const parsed = asRecord(result);
  const body = (() => {
    switch (name) {
      case "list_gaps":
        return <Rows rows={asRows(parsed.gaps)} primary="name" secondary="entity_id" />;
      case "list_matchers":
        return <Rows rows={asRows(parsed.matchers)} primary="id" secondary="enabled" />;
      case "list_policies":
        return <Rows rows={asRows(parsed.policies)} primary="label" secondary="id" />;
      case "list_languages":
        return <p className="trainer-card-tags">{asStrings(parsed.languages).join(" · ")}</p>;
      case "list_lexicon_paths":
        return <p className="trainer-card-tags">{asStrings(parsed.paths).slice(0, 16).join(" · ")}</p>;
      case "search_house":
        return (
          <>
            {parsedArgs.q ? <p className="muted">{String(parsedArgs.q)}</p> : null}
            <Rows rows={asRows(parsed.entities)} primary="name" secondary="entity_id" />
          </>
        );
      case "get_entity":
        return (
          <Rows
            rows={parsed.entity_id ? [parsed] : []}
            primary="name"
            secondary="entity_id"
          />
        );
      case "get_lexicon":
        return <p className="mono">{textOf(parsed, "path") || textOf(parsedArgs, "path")}</p>;
      case "validate_proposal":
        return <p>{parsed.ok === false ? "fail" : "ok"}</p>;
      case "apply_lexicon":
      case "apply_match":
      case "apply_house":
      case "apply_aliases":
        return <p className="muted">{result?.includes("error") ? result : "geschrieben"}</p>;
      default: {
        const _keep: string = name;
        return result ? <p className="mono">{_keep}</p> : null;
      }
    }
  })();

  return (
    <article className="trainer-card">
      <p className="trainer-kicker">{name}</p>
      {body}
    </article>
  );
}
