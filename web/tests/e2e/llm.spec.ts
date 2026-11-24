import http from "node:http";
import type { AddressInfo } from "node:net";
import { en } from "../../src/i18n/en";
import { expect, test } from "./engine";

// The stored LLM key stays with the stored address, checked end to end: a real engine, the built
// UI in a browser, and two stand-in LLM servers that record who asked them and with which key.

type Hit = { path: string; authorization: string; fromBrowser: boolean };
type StandIn = { url: string; hits: Hit[]; close: () => Promise<void> };

const STORED_KEY = "sk-made-up-stored-key";

/** An OpenAI-style server that answers every request with one model and allows any browser origin. */
async function standIn(model: string): Promise<StandIn> {
  const hits: Hit[] = [];
  const server = http.createServer((request, response) => {
    hits.push({
      path: request.url ?? "",
      authorization: request.headers.authorization ?? "",
      fromBrowser: Boolean(request.headers.origin),
    });
    response.writeHead(200, {
      "content-type": "application/json",
      "access-control-allow-origin": "*",
      "access-control-allow-headers": "*",
    });
    response.end(JSON.stringify({ data: [{ id: model }] }));
  });
  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", resolve));
  const { port } = server.address() as AddressInfo;
  return {
    url: `http://127.0.0.1:${port}/v1`,
    hits,
    close: () => new Promise((resolve) => server.close(() => resolve())),
  };
}

test("typing an LLM URL contacts nobody, and the stored key only ever reaches the stored address", async ({ page, engine }) => {
  const stored = await standIn("stored-model");
  const other = await standIn("other-model");
  try {
    await engine.finishSetup();
    await engine.api("/api/v2/llm/endpoint", { base_url: stored.url, model: "stored-model", api_key: STORED_KEY, configured: true, provider: "custom" });
    const modelRequests: string[] = [];
    page.on("request", (request) => {
      if (request.url().includes("/api/v2/llm/models")) modelRequests.push(request.url());
    });
    await page.goto(`${engine.url}/#/settings/llm`);
    const url = page.getByLabel(en.llmBaseUrl);
    await expect(url).toHaveValue(stored.url);

    // Opening the settings and typing another address, with a pause in the middle, asks nothing.
    await url.fill("");
    await url.pressSequentially(other.url.slice(0, -4), { delay: 40 });
    await page.waitForTimeout(700);
    await url.pressSequentially(other.url.slice(-4), { delay: 40 });
    await page.waitForTimeout(700);
    expect(modelRequests).toEqual([]);
    expect(stored.hits).toEqual([]);
    expect(other.hits).toEqual([]);

    // Opening the model list asks the engine, which asks the other address without the stored key.
    await page.getByLabel(en.llmModel).click();
    await expect(page.getByRole("option", { name: "other-model" })).toBeVisible();
    expect(other.hits.length).toBeGreaterThan(0);
    expect(other.hits.every((hit) => !hit.fromBrowser)).toBe(true);
    expect(other.hits.map((hit) => hit.authorization).filter(Boolean)).toEqual([]);
    expect(stored.hits).toEqual([]);

    // Back on the stored address the engine uses the stored key; the browser never holds it.
    await page.keyboard.press("Escape");
    await url.fill(stored.url);
    await page.getByLabel(en.llmModel).click();
    await expect(page.getByRole("option", { name: "stored-model" })).toBeVisible();
    expect(stored.hits.length).toBeGreaterThan(0);
    expect(stored.hits.every((hit) => !hit.fromBrowser && hit.authorization === `Bearer ${STORED_KEY}`)).toBe(true);
    expect(other.hits.map((hit) => hit.authorization).filter(Boolean)).toEqual([]);
  } finally {
    await stored.close();
    await other.close();
  }
});
