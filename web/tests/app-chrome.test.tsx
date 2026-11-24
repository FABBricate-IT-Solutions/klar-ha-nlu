import { screen, waitFor, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "../src/App";
import { de } from "../src/i18n/de";
import { en } from "../src/i18n/en";
import { fakeEngine } from "./support/fakeEngine";
import { storedSettings, storedUi } from "./support/fixtures";
import { renderUi } from "./support/render";

// The operator chrome (navigation, page titles) follows the UI language saved in the engine. The
// Assist language pin in the settings decides which language the Lab parses in, nothing else.

async function openMenu(user: ReturnType<typeof renderUi>["user"], label: string) {
  await user.click(await screen.findByRole("button", { name: label }));
  return screen.findByRole("navigation", { name: "Klar!" });
}

describe("the app chrome", () => {
  it("uses the saved UI language even when Assist is pinned to another language", async () => {
    fakeEngine({ stored: { settings: { ...storedSettings, languages: ["de"] }, ui: { ...storedUi, locale: "en" } } });
    const { user } = renderUi(<App />);
    await screen.findByRole("heading", { name: en.understandsHome });

    const nav = await openMenu(user, en.menu);

    expect(within(nav).getByRole("link", { name: en.settings })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("en");
    expect(document.documentElement.dir).toBe("ltr");
  });

  it("is German for a German UI even when Assist is pinned to English", async () => {
    fakeEngine({ stored: { settings: { ...storedSettings, languages: ["en"] }, ui: { ...storedUi, locale: "de" } } });
    const { user } = renderUi(<App />);
    await screen.findByRole("heading", { name: de.understandsHome });

    const nav = await openMenu(user, de.menu);

    expect(within(nav).getByRole("link", { name: de.settings })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("de");
  });

  it("does not follow the browser language when no UI language is saved", async () => {
    vi.spyOn(window.navigator, "language", "get").mockReturnValue("de-DE");
    vi.spyOn(window.navigator, "languages", "get").mockReturnValue(["de-DE", "de"]);
    fakeEngine({ stored: { ui: { ...storedUi, locale: "", locale_set: false } } });
    renderUi(<App />);

    expect(await screen.findByRole("heading", { name: en.understandsHome })).toBeInTheDocument();
    expect(document.documentElement.lang).toBe("en");
  });

  it("lays the page out right to left for an RTL UI language", async () => {
    fakeEngine({ stored: { ui: { ...storedUi, locale: "ar" } } });
    renderUi(<App />);

    await waitFor(() => expect(document.documentElement.dir).toBe("rtl"));
    expect(document.documentElement.lang).toBe("ar");
  });

  it("opens the navigation from the menu button and closes it on a link", async () => {
    fakeEngine();
    const { user } = renderUi(<App />);
    const menu = await screen.findByRole("button", { name: en.menu });
    expect(menu).toHaveAttribute("aria-expanded", "false");
    expect(screen.queryByRole("navigation", { name: "Klar!" })).not.toBeInTheDocument();

    const nav = await openMenu(user, en.menu);

    expect(menu).toHaveAttribute("aria-expanded", "true");
    expect(menu.getAttribute("aria-controls")).toBe(nav.closest("[id]")?.id);
    const links = within(nav).getAllByRole("link").map((link) => [link.textContent, link.getAttribute("href")]);
    expect(links).toEqual([
      [en.home, "#/"],
      [en.conversations, "#/conversations"],
      [en.rules, "#/rules/routines"],
      [en.house, "#/house/mapping"],
      [en.lab, "#/lab"],
      [en.settings, "#/settings/llm"],
    ]);
    expect(within(nav).getByRole("link", { name: en.home })).toHaveAttribute("aria-current", "page");

    await user.click(within(nav).getByRole("link", { name: en.lab }));

    await waitFor(() => expect(screen.queryByRole("navigation", { name: "Klar!" })).not.toBeInTheDocument());
  });

  it("opens the page a deep link names", async () => {
    fakeEngine();
    window.history.replaceState(null, "", "/#/lab");
    renderUi(<App />);

    expect(await screen.findByRole("heading", { name: en.lab })).toBeInTheDocument();
  });

  it("parses in the pinned Assist language on the Lab page, not in the UI language", async () => {
    const engine = fakeEngine({ stored: { settings: { ...storedSettings, languages: ["de"] }, ui: { ...storedUi, locale: "en" } } });
    window.history.replaceState(null, "", "/#/lab");
    const { user } = renderUi(<App />);

    await user.click(await screen.findByRole("button", { name: en.analyze }));

    await waitFor(() => expect(engine.to("POST /api/v2/parse")).toHaveLength(1));
    expect(engine.to("POST /api/v2/parse")[0].body).toMatchObject({ language: "de" });
  });

  it("parses in the UI language on the Lab page when Assist listens to several languages", async () => {
    const engine = fakeEngine({ stored: { settings: { ...storedSettings, languages: ["de", "en"] }, ui: { ...storedUi, locale: "en" } } });
    window.history.replaceState(null, "", "/#/lab");
    const { user } = renderUi(<App />);

    await user.click(await screen.findByRole("button", { name: en.analyze }));

    await waitFor(() => expect(engine.to("POST /api/v2/parse")).toHaveLength(1));
    expect(engine.to("POST /api/v2/parse")[0].body).toMatchObject({ language: "en" });
  });
});
