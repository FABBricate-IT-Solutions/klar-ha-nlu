import { useState } from "react";
import { writeCustomVoice, type LlmChatTarget } from "../customVoice";
import type { Messages } from "../i18n";
import type { Settings, VoiceTraits } from "../types";
import { readTraits, TRAIT_KEYS, type TraitKey } from "../voiceTraits";
import { Button } from "@/components/ui/button";
import { Field, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Slider } from "@/components/ui/slider";
import { Textarea } from "@/components/ui/textarea";
import { ToggleGroup, ToggleGroupItem } from "@/components/ui/toggle-group";

function traitLabel(t: Messages, key: TraitKey): string {
  switch (key) {
    case "warmth":
      return t.interviewWarmth;
    case "humor":
      return t.interviewHumor;
    case "sarcasm":
      return t.interviewSarcasm;
    case "formality":
      return t.interviewFormality;
    case "verbosity":
      return t.interviewVerbosity;
    case "energy":
      return t.interviewEnergy;
    default: {
      const _never: never = key;
      return _never;
    }
  }
}

export function CustomVoiceInterview({
  t,
  language,
  settings,
  llm,
  onSettings,
}: {
  t: Messages;
  language: string;
  settings: Settings;
  llm?: LlmChatTarget;
  onSettings: (next: Settings) => void;
}) {
  const [address, setAddress] = useState<"du" | "sie" | "name">("du");
  const [operatorName, setOperatorName] = useState("");
  const [taboo, setTaboo] = useState("");
  const [status, setStatus] = useState("");
  const [busy, setBusy] = useState(false);
  const traits = readTraits(settings.custom_voice_traits);
  const voiceName = settings.custom_voice_name || "";
  const seed = settings.custom_voice_seed || "";

  const patch = (next: Partial<Settings>) => {
    onSettings({ ...settings, personality: "custom", ...next });
  };

  const setTrait = (key: TraitKey, value: number) => {
    const next: VoiceTraits = { ...traits, [key]: value };
    patch({ custom_voice_traits: next });
  };

  const write = async () => {
    setBusy(true);
    setStatus("");
    try {
      const prompt = await writeCustomVoice(
        {
          language,
          address,
          name: operatorName,
          voice_name: voiceName,
          seed,
          ...traits,
          taboo,
        },
        llm,
      );
      patch({ custom_voice: prompt });
      setStatus(t.customVoiceMake);
    } catch {
      setStatus(t.customVoiceFail);
    } finally {
      setBusy(false);
    }
  };

  return (
    <FieldGroup>
      <Field>
        <FieldLabel htmlFor="custom-voice-title">{t.customVoiceName}</FieldLabel>
        <Input
          id="custom-voice-title"
          value={voiceName}
          onChange={(ev) => patch({ custom_voice_name: ev.target.value })}
        />
        <FieldDescription>{t.customVoiceNameHint}</FieldDescription>
      </Field>
      <Field>
        <FieldLabel htmlFor="custom-voice-seed">{t.customVoiceSeed}</FieldLabel>
        <Textarea
          id="custom-voice-seed"
          value={seed}
          onChange={(ev) => patch({ custom_voice_seed: ev.target.value })}
        />
        <FieldDescription>{t.customVoiceSeedHint}</FieldDescription>
      </Field>
      <Field>
        <FieldLabel>{t.interviewAddress}</FieldLabel>
        <ToggleGroup
          variant="outline"
          spacing={0}
          value={[address]}
          onValueChange={(next) => {
            const value = next[0];
            if (value === "du" || value === "sie" || value === "name") setAddress(value);
          }}
          aria-label={t.interviewAddress}
        >
          <ToggleGroupItem value="du">{t.interviewAddressDu}</ToggleGroupItem>
          <ToggleGroupItem value="sie">{t.interviewAddressSie}</ToggleGroupItem>
          <ToggleGroupItem value="name">{t.interviewAddressName}</ToggleGroupItem>
        </ToggleGroup>
      </Field>
      {address === "name" ? (
        <Field>
          <FieldLabel htmlFor="custom-voice-operator">{t.interviewName}</FieldLabel>
          <Input id="custom-voice-operator" value={operatorName} onChange={(ev) => setOperatorName(ev.target.value)} />
        </Field>
      ) : null}
      {TRAIT_KEYS.map((key) => (
        <Field key={key}>
          <div className="flex items-center justify-between gap-2">
            <FieldLabel htmlFor={`custom-voice-${key}`}>{traitLabel(t, key)}</FieldLabel>
            <span className="font-mono text-xs text-muted-foreground">{traits[key]}</span>
          </div>
          <Slider
            id={`custom-voice-${key}`}
            min={0}
            max={10}
            step={1}
            value={[traits[key]]}
            onValueChange={(next) => {
              const value = Array.isArray(next) ? next[0] : next;
              if (typeof value === "number") setTrait(key, value);
            }}
          />
        </Field>
      ))}
      <FieldDescription>{t.interviewTraitsHint}</FieldDescription>
      <Field>
        <FieldLabel htmlFor="custom-voice-taboo">{t.interviewTaboo}</FieldLabel>
        <Input id="custom-voice-taboo" value={taboo} onChange={(ev) => setTaboo(ev.target.value)} />
      </Field>
      <Button type="button" variant="outline" disabled={busy} onClick={() => void write()}>
        {t.customVoiceMake}
      </Button>
      {settings.custom_voice ? (
        <pre className="caption max-w-full overflow-x-auto whitespace-pre-wrap">{settings.custom_voice}</pre>
      ) : null}
      {status ? <p className="caption">{status}</p> : null}
      <p className="caption">{t.customVoiceHint}</p>
    </FieldGroup>
  );
}
