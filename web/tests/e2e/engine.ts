import { spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test as base, expect } from "@playwright/test";

const webDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");
const repoDir = path.resolve(webDir, "..");

export type Engine = {
  /** Origin of the engine, for example `http://127.0.0.1:41873`. */
  url: string;
  /** GET (no body) or POST (with body) against the engine's API, from the test process. */
  api: <T>(pathname: string, body?: unknown) => Promise<T>;
  /** Marks first-run setup as done so the wizard stays closed. */
  finishSetup: () => Promise<void>;
};

/** `KLAR_BIN`, else the debug binary in `CARGO_TARGET_DIR` or the repository's `target/`. */
function engineBinary(): string {
  const candidates = [
    process.env.KLAR_BIN,
    process.env.CARGO_TARGET_DIR && path.join(process.env.CARGO_TARGET_DIR, "debug", "klar"),
    path.join(repoDir, "target", "debug", "klar"),
  ].filter((candidate): candidate is string => Boolean(candidate));
  const found = candidates.find((candidate) => fs.existsSync(candidate));
  if (!found) {
    throw new Error(`No engine binary. Run "cargo build --locked" or set KLAR_BIN. Looked at: ${candidates.join(", ")}`);
  }
  return found;
}

function freePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      server.close(() => (address && typeof address === "object" ? resolve(address.port) : reject(new Error("no port"))));
    });
  });
}

async function waitUntilUp(url: string, child: ChildProcess, log: () => string): Promise<void> {
  const deadline = Date.now() + 20_000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`The engine exited with code ${child.exitCode}:\n${log()}`);
    try {
      if ((await fetch(`${url}/api/settings`)).ok) return;
    } catch {
      // not listening yet
    }
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`The engine did not answer on ${url} within 20 s:\n${log()}`);
}

export const test = base.extend<{ engine: Engine }>({
  // Playwright reads the fixture's dependencies from this destructuring pattern; there are none.
  // eslint-disable-next-line no-empty-pattern
  engine: async ({}, provide) => {
    const ui = path.join(webDir, "dist");
    if (!fs.existsSync(path.join(ui, "index.html"))) throw new Error('web/dist is missing: run "npm run build" first.');
    const home = fs.mkdtempSync(path.join(os.tmpdir(), "klar-e2e-"));
    fs.mkdirSync(path.join(home, "config"));
    fs.mkdirSync(path.join(home, "data"));
    const [http, wyoming] = [await freePort(), await freePort()];
    const url = `http://127.0.0.1:${http}`;
    let output = "";
    const child = spawn(
      engineBinary(),
      ["--http", `127.0.0.1:${http}`, "--wyoming", `127.0.0.1:${wyoming}`, "--config-dir", path.join(home, "config"), "--data-dir", path.join(home, "data")],
      { env: { ...process.env, KLAR_UI_DIR: ui }, stdio: ["ignore", "pipe", "pipe"] },
    );
    child.stdout?.on("data", (chunk: Buffer) => (output += chunk.toString()));
    child.stderr?.on("data", (chunk: Buffer) => (output += chunk.toString()));
    try {
      await waitUntilUp(url, child, () => output);
      const api = async <T>(pathname: string, body?: unknown): Promise<T> => {
        const response = await fetch(`${url}${pathname}`, body === undefined ? {} : {
          method: "POST",
          headers: { "content-type": "application/json" },
          body: JSON.stringify(body),
        });
        expect(response.ok, `${pathname} answered ${response.status}`).toBe(true);
        return (await response.json()) as T;
      };
      await provide({
        url,
        api,
        finishSetup: async () => {
          const state = await api<Record<string, unknown>>("/api/ui");
          await api("/api/ui", { ...state, wizard_done: true });
        },
      });
    } finally {
      child.kill();
      await new Promise((resolve) => (child.exitCode === null ? child.once("exit", resolve) : resolve(undefined)));
      fs.rmSync(home, { recursive: true, force: true });
    }
  },
});

export { expect };
