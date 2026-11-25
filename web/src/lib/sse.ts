/**
 * Split an SSE buffer into complete events (blank-line delimited) and leftover bytes.
 * Each event may carry several `data:` lines; they are joined with newlines (SSE spec).
 */
export function takeSseEvents(buf: string): { events: string[]; rest: string } {
  const chunks = buf.split("\n\n");
  const rest = chunks.pop() ?? "";
  const events: string[] = [];
  for (const chunk of chunks) {
    const payloads = chunk
      .split("\n")
      .filter((row) => row.startsWith("data:"))
      .map((row) => row.slice("data:".length).replace(/^\s/, ""));
    if (payloads.length === 0) continue;
    const payload = payloads.join("\n").trim();
    if (!payload || payload === "[DONE]") continue;
    events.push(payload);
  }
  return { events, rest };
}
