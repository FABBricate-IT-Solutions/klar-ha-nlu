import type { Page } from "@playwright/test";
import { en } from "../../src/i18n/en";
import { expect, test } from "./engine";

// Smoke test: the built UI, served by a real engine, in a real browser. It covers what jsdom
// cannot: the engine's own answers, the bundle as it ships, and layout.

type Settings = { nlu_rag: boolean; languages: string[] };
type UiState = { wizard_done: boolean; locale: string };
type Policies = { policies: { id: string; label: string; enabled: boolean }[] };

// The page loads its typeface from Google Fonts (theme.css). That is the one known reason for the
// operator's browser to contact another host; any other host showing up here is a regression.
const FONT_HOSTS = ["fonts.googleapis.com", "fonts.gstatic.com"];

/** Collects what a healthy page must not produce: script errors and requests to an unexpected host. */
function watch(page: Page, origin: string) {
  const errors: string[] = [];
  const foreign = new Set<string>();
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (url.protocol !== "http:" && url.protocol !== "https:") return;
    if (url.origin !== origin && !FONT_HOSTS.includes(url.hostname)) foreign.add(url.origin);
  });
  return { errors, foreign };
}

test("a fresh engine greets with the setup wizard and remembers that it was finished", async ({ page, engine }) => {
  const seen = watch(page, engine.url);
  await page.goto(engine.url);

  const wizard = page.getByRole("dialog");
  await expect(wizard).toBeVisible();
  await expect(wizard.getByRole("button", { name: "1", exact: true })).toHaveAttribute("aria-current", "step");
  await wizard.getByRole("button", { name: "6", exact: true }).click();
  await wizard.getByRole("button", { name: "Done", exact: true }).click();
  await expect(wizard).toBeHidden();

  await expect.poll(async () => (await engine.api<UiState>("/api/ui")).wizard_done).toBe(true);
  await page.reload();
  await expect(page.getByRole("heading", { name: en.understandsHome })).toBeVisible();
  await expect(page.getByRole("dialog")).toBeHidden();
  expect(seen.errors).toEqual([]);
  expect([...seen.foreign]).toEqual([]);
});

test("a changed setting reaches the engine and survives a reload", async ({ page, engine }) => {
  await engine.finishSetup();
  const before = await engine.api<Settings>("/api/settings");
  expect(before.nlu_rag).toBe(false);
  await page.goto(`${engine.url}/#/settings/engine`);

  const toggle = page.getByRole("switch", { name: en.nluRag });
  await expect(toggle).toHaveAttribute("aria-checked", "false");
  await toggle.click();
  await page.getByRole("button", { name: en.save, exact: true }).click();
  await expect(page.getByText(en.saveOk)).toBeVisible();

  expect((await engine.api<Settings>("/api/settings")).nlu_rag).toBe(true);
  await page.reload();
  await expect(page.getByRole("switch", { name: en.nluRag })).toHaveAttribute("aria-checked", "true");
});

test("the Lab parses a sentence with the engine and shows what it would do", async ({ page, engine }) => {
  await engine.finishSetup();
  const seen = watch(page, engine.url);
  await page.goto(`${engine.url}/#/lab`);

  await page.getByLabel(en.command).fill("turn on the kitchen light");
  await page.getByRole("button", { name: en.analyze, exact: true }).click();

  const path = page.getByLabel(en.labThisTurn);
  await expect(path).toContainText("HassTurnOn");
  await expect(page.locator(".page.lab")).toHaveAttribute("data-band", "execute");
  await expect(page.getByRole("alert")).toHaveCount(0);
  expect(seen.errors).toEqual([]);
  expect([...seen.foreign]).toEqual([]);
});

test("a new house rule is stored by the engine and the rule lanes scroll inside themselves", async ({ page, engine }) => {
  await engine.finishSetup();
  expect((await engine.api<Policies>("/api/v2/policies")).policies).toEqual([]);
  await page.goto(`${engine.url}/#/rules/policies`);

  await page.getByRole("button", { name: en.addRule, exact: true }).click();
  await page.getByRole("button", { name: en.save, exact: true }).click();
  await expect(page.getByText(en.saveOk).first()).toBeVisible();

  const stored = await engine.api<Policies>("/api/v2/policies");
  expect(stored.policies).toHaveLength(1);
  expect(stored.policies[0].enabled).toBe(true);

  // Long lists must scroll inside their lane instead of stretching the page.
  const lanes = page.locator(".policy-lane .lane-body");
  await expect(lanes).toHaveCount(3);
  for (const lane of await lanes.all()) {
    const style = await lane.evaluate((node) => {
      const computed = getComputedStyle(node);
      return { overflowY: computed.overflowY };
    });
    expect(style.overflowY).toBe("auto");
  }
});

test("the navigation menu opens on a phone-sized screen and leads to Settings", async ({ page, engine }) => {
  await engine.finishSetup();
  await page.setViewportSize({ width: 390, height: 800 });
  await page.goto(engine.url);

  await page.getByRole("button", { name: en.menu, exact: true }).click();
  await page.getByRole("navigation", { name: "Klar!" }).getByRole("link", { name: en.settings, exact: true }).click();

  await expect(page.getByRole("heading", { name: en.settings, exact: true })).toBeVisible();
  expect(new URL(page.url()).hash).toBe("#/settings/llm");
});
