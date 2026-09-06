import { api } from "./api";
import { modelAuthHeaders } from "./llmModels";
import type { LlmProviderId } from "./llmProviders";

const MAX_EXTRA = 2048;
const SYSTEM =
  "You write a Klar NLU voice block only. " +
  "If a seed character is given, keep that identity. " +
  "Sliders 0-10 only refine delivery (warmth, humor, sarcasm, formality, verbosity, energy). " +
  "They must not replace or ignore the seed. " +
  "Output the voice description and a few short example rewrites (source → spoken). " +
  "Keep language lock and safety: no Home Assistant tools, no device control, " +
  "digits stay digits, no new facts. Do not repeat the safety rules. " +
  "Do not mention this instruction. No markdown fences. No title.";

export type CustomVoiceIn = {
  language: string;
  address: string;
  name?: string;
  voice_name?: string;
  seed?: string;
  warmth: number;
  humor: number;
  sarcasm: number;
  formality: number;
  verbosity: number;
  energy: number;
  taboo?: string;
};

export type LlmChatTarget = {
  baseUrl: string;
  model: string;
  apiKey: string;
  provider: LlmProviderId;
};

export async function writeCustomVoice(body: CustomVoiceIn, llm?: LlmChatTarget): Promise<string> {
  try {
    const out = await api.customVoice(body);
    if (out.prompt.trim()) return clipVoice(out.prompt);
  } catch {
    if (!llm?.baseUrl.trim() || !llm.model.trim()) throw new Error("voice");
  }
  if (!llm?.baseUrl.trim() || !llm.model.trim()) throw new Error("voice");
  return clipVoice(await chatDirect(llm, body));
}

function languageLock(pack: string): string {
  const tag = pack.trim().toLowerCase();
  if (tag === "de" || tag.startsWith("de-")) {
    return "Antworte nur auf Deutsch. Übersetze nicht ins Englische oder in eine andere Sprache.";
  }
  if (tag === "en" || tag.startsWith("en-")) {
    return "Answer only in English. Do not translate into German or any other language.";
  }
  return `Answer only in the Klar NLU pack ${pack}. Do not translate into German, English, or any other language.`;
}

function interviewLine(body: CustomVoiceIn): string {
  const address =
    body.address === "name" ? `address by first name (${(body.name || "").trim()})` : body.address;
  const voiceName = (body.voice_name || "").trim() || "custom";
  let line =
    `Voice name: ${voiceName}.\nLanguage pack: ${body.language}.\nAddress the operator: ${address}.\n` +
    `Traits 0-10 (delivery only): warmth=${body.warmth}, humor=${body.humor}, sarcasm=${body.sarcasm}, ` +
    `formality=${body.formality}, verbosity=${body.verbosity}, energy=${body.energy}.`;
  const seed = (body.seed || "").trim();
  if (seed) line += `\nSeed character (keep this identity; sliders only refine delivery):\n${seed}`;
  const taboo = (body.taboo || "").trim();
  if (taboo) line += `\nDo not say: ${taboo}`;
  return line;
}

function chatUrls(baseUrl: string): string[] {
  const base = baseUrl.trim().replace(/\/+$/, "");
  const urls = [`${base}/chat/completions`];
  if (base.endsWith("/api/v1")) {
    urls.push(`${base.slice(0, -"/api/v1".length)}/v1/chat/completions`);
  } else if (base.endsWith("/v1")) {
    urls.push(`${base.slice(0, -"/v1".length)}/api/v1/chat/completions`);
  }
  return [...new Set(urls)];
}

function noThinking(provider: LlmProviderId): Record<string, unknown> {
  switch (provider) {
    case "openai":
      return {};
    case "anthropic":
      return {};
    case "google":
      return {
        reasoning_effort: "none",
        extra_body: { google: { thinking_config: { thinking_budget: 0 } } },
      };
    default:
      return { chat_template_kwargs: { enable_thinking: false } };
  }
}

function completionText(value: unknown): string {
  if (!value || typeof value !== "object") return "";
  const choices = (value as { choices?: unknown }).choices;
  if (!Array.isArray(choices) || !choices[0] || typeof choices[0] !== "object") return "";
  const message = (choices[0] as { message?: { content?: unknown } }).message;
  const content = message?.content;
  return typeof content === "string" ? content : "";
}

async function chatDirect(llm: LlmChatTarget, body: CustomVoiceIn): Promise<string> {
  const key = llm.apiKey.trim() || (llm.provider === "lemonade" ? "lemonade" : "");
  const headers: Record<string, string> = {
    ...(modelAuthHeaders(llm.provider, key) as Record<string, string>),
    "Content-Type": "application/json",
  };
  const payload = {
    model: llm.model.trim(),
    messages: [
      { role: "system", content: `${SYSTEM}\n\n${languageLock(body.language)}` },
      { role: "user", content: interviewLine(body) },
    ],
    stream: false,
    temperature: 0.4,
    max_tokens: 512,
    ...noThinking(llm.provider),
  };
  for (const url of chatUrls(llm.baseUrl)) {
    try {
      const response = await fetch(url, { method: "POST", headers, body: JSON.stringify(payload) });
      if (!response.ok) continue;
      const text = completionText(await response.json());
      if (text.trim()) return text;
    } catch {
      continue;
    }
  }
  throw new Error("voice");
}

function clipVoice(raw: string): string {
  const prompt = raw.trim();
  if (!prompt || [...prompt].length > MAX_EXTRA) throw new Error("voice");
  return prompt;
}
