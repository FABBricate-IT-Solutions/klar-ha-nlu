import type { ReactNode } from "react";

function inline(text: string): ReactNode[] {
  const chunks = text.split(/(\*\*[^*]+\*\*|`[^`]+`)/g);
  return chunks.map((chunk, index) => {
    if (chunk.startsWith("**") && chunk.endsWith("**") && chunk.length > 4) {
      return <strong key={index}>{chunk.slice(2, -2)}</strong>;
    }
    if (chunk.startsWith("`") && chunk.endsWith("`") && chunk.length > 2) {
      return <code key={index}>{chunk.slice(1, -1)}</code>;
    }
    return <span key={index}>{chunk}</span>;
  });
}

function isListLine(line: string): boolean {
  return /^\s*(?:[-*]|\d+\.)\s+/.test(line);
}

function listItem(line: string): string {
  return line.replace(/^\s*(?:[-*]|\d+\.)\s+/, "");
}

export function TrainerMarkdown({ text }: { text: string }) {
  if (!text.trim()) {
    return null;
  }
  const blocks = text.replaceAll("\r\n", "\n").split(/\n{2,}/);
  return (
    <div className="trainer-md">
      {blocks.map((block, index) => {
        const lines = block.split("\n").filter((line) => line.trim());
        if (lines.length > 0 && lines.every(isListLine)) {
          return (
            <ul className="trainer-md-list" key={index}>
              {lines.map((line, row) => (
                <li key={`${index}-${row}`}>{inline(listItem(line))}</li>
              ))}
            </ul>
          );
        }
        const heading = lines[0]?.match(/^(#{1,3})\s+(.*)$/);
        if (heading) {
          return <p className="trainer-md-head" key={index}>{inline(heading[2] ?? "")}</p>;
        }
        return <p key={index}>{inline(lines.join(" "))}</p>;
      })}
    </div>
  );
}
