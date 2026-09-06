import { useEffect, useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";

export function PersonalityPrompt({
  t,
  personality,
  language,
}: {
  t: Messages;
  personality: string;
  language: string;
}) {
  const [flavor, setFlavor] = useState("");
  const [prompt, setPrompt] = useState("");
  useEffect(() => {
    api.llmVoice(personality, language)
      .then((next) => {
        setFlavor(next.flavor);
        setPrompt(next.prompt);
      })
      .catch(() => {
        setFlavor("");
        setPrompt("");
      });
  }, [personality, language]);
  if (!prompt) return null;
  return (
    <Field>
      <FieldLabel>{t.personalityPrompt}</FieldLabel>
      {flavor ? <p className="text-sm text-muted-foreground">{flavor}</p> : null}
      <pre className="max-h-48 overflow-auto whitespace-pre-wrap rounded-md border border-border bg-muted/40 p-3 font-mono text-xs">
        {prompt}
      </pre>
      <FieldDescription>{t.personalityPromptHint}</FieldDescription>
    </Field>
  );
}
