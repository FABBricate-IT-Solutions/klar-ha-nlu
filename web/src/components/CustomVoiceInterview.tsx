import { useState } from "react";
import { api } from "../api";
import type { Messages } from "../i18n";

const field = { display: "block", marginTop: 12 } as const;
const control = { display: "block", minHeight: 44, width: "100%", marginTop: 6 } as const;

export function CustomVoiceInterview({
  t,
  language,
  value,
  onChange,
}: {
  t: Messages;
  language: string;
  value: string;
  onChange: (prompt: string) => void;
}) {
  const [address, setAddress] = useState<"du" | "sie" | "name">("du");
  const [name, setName] = useState("");
  const [tone, setTone] = useState<"short" | "warm" | "dry">("warm");
  const [humor, setHumor] = useState<"none" | "light" | "sharp">("light");
  const [length, setLength] = useState<"one" | "more">("one");
  const [taboo, setTaboo] = useState("");
  const [status, setStatus] = useState("");

  const write = async () => {
    try {
      const out = await api.customVoice({
        language,
        address,
        name,
        tone,
        humor,
        length,
        taboo,
      });
      onChange(out.prompt);
      setStatus(t.customVoiceMake);
    } catch {
      setStatus(t.customVoiceFail);
    }
  };

  return (
    <div>
      <label style={field}>
        {t.interviewAddress}
        <select style={control} value={address} onChange={(ev) => setAddress(ev.target.value as "du" | "sie" | "name")}>
          <option value="du">{t.interviewAddressDu}</option>
          <option value="sie">{t.interviewAddressSie}</option>
          <option value="name">{t.interviewAddressName}</option>
        </select>
      </label>
      {address === "name" ? (
        <label style={field}>
          {t.interviewName}
          <input style={control} value={name} onChange={(ev) => setName(ev.target.value)} />
        </label>
      ) : null}
      <label style={field}>
        {t.interviewTone}
        <select style={control} value={tone} onChange={(ev) => setTone(ev.target.value as "short" | "warm" | "dry")}>
          <option value="short">{t.interviewToneShort}</option>
          <option value="warm">{t.interviewToneWarm}</option>
          <option value="dry">{t.interviewToneDry}</option>
        </select>
      </label>
      <label style={field}>
        {t.interviewHumor}
        <select style={control} value={humor} onChange={(ev) => setHumor(ev.target.value as "none" | "light" | "sharp")}>
          <option value="none">{t.interviewHumorNone}</option>
          <option value="light">{t.interviewHumorLight}</option>
          <option value="sharp">{t.interviewHumorSharp}</option>
        </select>
      </label>
      <label style={field}>
        {t.interviewLength}
        <select style={control} value={length} onChange={(ev) => setLength(ev.target.value as "one" | "more")}>
          <option value="one">{t.interviewLengthOne}</option>
          <option value="more">{t.interviewLengthMore}</option>
        </select>
      </label>
      <label style={field}>
        {t.interviewTaboo}
        <input style={control} value={taboo} onChange={(ev) => setTaboo(ev.target.value)} />
      </label>
      <button type="button" className="secondary" style={{ minHeight: 44, marginTop: 12 }} onClick={() => void write()}>
        {t.customVoiceMake}
      </button>
      {value ? <pre className="caption" style={{ whiteSpace: "pre-wrap", marginTop: 12 }}>{value}</pre> : null}
      {status ? <p className="caption">{status}</p> : null}
      <p className="caption">{t.customVoiceHint}</p>
    </div>
  );
}
