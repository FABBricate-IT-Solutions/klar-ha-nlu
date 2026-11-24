import { describe, expect, it, vi } from "vitest";
import { assistParseLanguage, chromeLocale, dictionaries, fill, inboxReason, isLocale, isRtl, matchLocale } from "../src/i18n";
import { de } from "../src/i18n/de";
import { en } from "../src/i18n/en";
import { fillWizard, wizardMessages } from "../src/i18n/wizard";

describe("the UI language", () => {
  it("is the saved one when the UI has it", () => {
    expect(chromeLocale("de")).toBe("de");
    expect(chromeLocale("pt-BR")).toBe("pt-BR");
  });

  it("falls back to English, never to the browser language", () => {
    vi.spyOn(window.navigator, "language", "get").mockReturnValue("de-DE");

    expect(chromeLocale(undefined)).toBe("en");
    expect(chromeLocale("")).toBe("en");
    expect(chromeLocale("tlh")).toBe("en");
  });

  it("matches tags by case, separator, script and region", () => {
    expect(matchLocale("DE")).toBe("de");
    expect(matchLocale("pt_br")).toBe("pt-BR");
    expect(matchLocale("fr-CA")).toBe("fr");
    expect(matchLocale("zh")).toBe("zh-CN");
    expect(matchLocale("zh-Hant-TW")).toBe("zh-TW");
    expect(matchLocale("zh-HK")).toBe("zh-HK");
    expect(matchLocale("sr-Latn-RS")).toBe("sr-Latn");
    expect(matchLocale("tlh")).toBeUndefined();
    expect(matchLocale(undefined)).toBeUndefined();
  });

  it("knows which languages are written right to left", () => {
    expect(["ar", "he", "fa", "ur"].map(isRtl)).toEqual([true, true, true, true]);
    expect(isRtl("de")).toBe(false);
  });

  it("has a full dictionary for every language it offers", () => {
    const keys = Object.keys(en).sort();
    expect(Object.keys(de).sort()).toEqual(keys);
    expect(Object.keys(dictionaries).length).toBeGreaterThan(60);
    for (const [code, messages] of Object.entries(dictionaries)) {
      expect(Object.keys(messages).sort(), code).toEqual(keys);
      expect(Object.values(messages).every((text) => typeof text === "string" && text.length > 0), code).toBe(true);
      expect(isLocale(code)).toBe(true);
    }
    expect(isLocale("tlh")).toBe(false);
  });
});

describe("the language the Lab parses in", () => {
  it("is the pinned Assist language when exactly one is pinned", () => {
    expect(assistParseLanguage(["de"], "en")).toBe("de");
  });

  it("is the UI language when Assist listens to all or to several languages", () => {
    expect(assistParseLanguage([], "en")).toBe("en");
    expect(assistParseLanguage(["de", "fr"], "en")).toBe("en");
    expect(assistParseLanguage([], undefined)).toBeUndefined();
  });
});

describe("message templates", () => {
  it("fills every occurrence of a slot and leaves unknown slots alone", () => {
    expect(fill("{count} of {count} in {room}", { count: "3" })).toBe("3 of 3 in {room}");
    expect(fill(en.applyDone, { count: "2" })).toContain("2");
    expect(fill(en.applyDone, { count: "2" })).not.toContain("{count}");
  });

  it("fills the wizard step counter", () => {
    const copy = wizardMessages("en");

    expect(fillWizard(copy.stepOf, { n: "2", total: "6" })).toMatch(/2.*6/);
    expect(wizardMessages("tlh").next).toBe(copy.next);
  });

  it("translates the reasons a device sits in the inbox", () => {
    expect(inboxReason("missing_area", en)).toBe(en.reasonMissingArea);
    expect(inboxReason("weak_name", de)).toBe(de.reasonWeakName);
    expect(inboxReason("match:kitchen", en)).toBe(`${en.reasonMatch}: kitchen`);
    expect(inboxReason("something_new", en)).toBe("something_new");
  });
});

describe("the English copy", () => {
  it("describes the Lab as the Assist path and sentence triggers as the fallback", () => {
    expect(en.parseHint).toContain("Lab is the Assist path for the selected language");
    expect(en.parseHint).toContain("Sentence triggers run only if Klar is unreachable");
    expect(en.triggerFirst).not.toContain("trigger, then Klar, then intent_script");
    expect(Object.values(en).some((text) => text.includes("when this parse is not execute"))).toBe(false);
  });

  it("names the Lab stages the way the pipeline does", () => {
    expect([en.labParse, en.labChipLlmRefine, en.labChipCalendarLlm, en.labChipQuietAck, en.labChipLlmTools, en.labChipNluRag, en.labChipLlmChat]).toEqual([
      "Klar parse",
      "LLM refine",
      "calendar LLM",
      "quiet ack",
      "LLM tools",
      "NLU-RAG",
      "LLM chat",
    ]);
  });
});
