// Bundle budget for the built UI (run after `npm run build`). Package 2.17 splits language
// packs out of the main chunk: first paint must stay under 500 kB gzip (plan acceptance).
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const KB = 1024;
const BUDGET = {
  // First paint only (plan acceptance: under 500 kB gzip). Packs + wizard load later.
  entry: { raw: 1200 * KB, gzip: 500 * KB },
  // All JS including on-demand locale/wizard chunks. Raised vs the old single-chunk
  // 2947/790 budget because the split adds many small files; first load is gated above.
  // 6.7: live/history chunk + short English FALLBACK keys in every chrome pack (parity CI).
  // First-paint entry stays gated above; packs stay shared FALLBACKS, not unique translations.
  js: { raw: 4050 * KB, gzip: 1425 * KB },
  css: { raw: 128 * KB, gzip: 22 * KB },
};

const assets = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "dist", "assets");
if (!fs.existsSync(assets)) {
  console.error("web/dist/assets is missing: run `npm run build` first.");
  process.exit(1);
}

const kb = (bytes) => `${(bytes / KB).toFixed(1)} kB`;
let failed = false;

const indexHtml = path.resolve(assets, "..", "index.html");
const html = fs.existsSync(indexHtml) ? fs.readFileSync(indexHtml, "utf8") : "";
const entryMatch = html.match(/assets\/(index-[^"']+\.js)/);
const entryName = entryMatch?.[1];
if (!entryName) {
  console.error("entry: could not find the index-*.js script in dist/index.html");
  failed = true;
} else {
  const buffer = fs.readFileSync(path.join(assets, entryName));
  const raw = buffer.length;
  const gzip = gzipSync(buffer, { level: 9 }).length;
  const over = raw > BUDGET.entry.raw || gzip > BUDGET.entry.gzip;
  failed ||= over;
  console.log(
    `${over ? "OVER " : "ok   "}entry ${entryName}  raw ${kb(raw)} of ${kb(BUDGET.entry.raw)}  gzip ${kb(gzip)} of ${kb(BUDGET.entry.gzip)}`,
  );
}

for (const [kind, limit] of Object.entries({ js: BUDGET.js, css: BUDGET.css })) {
  const files = fs.readdirSync(assets).filter((name) => name.endsWith(`.${kind}`));
  if (files.length === 0) {
    console.error(`${kind}: no file in dist/assets, the build output changed shape.`);
    failed = true;
    continue;
  }
  const contents = files.map((name) => fs.readFileSync(path.join(assets, name)));
  const raw = contents.reduce((sum, buffer) => sum + buffer.length, 0);
  const gzip = contents.reduce((sum, buffer) => sum + gzipSync(buffer, { level: 9 }).length, 0);
  const over = raw > limit.raw || gzip > limit.gzip;
  failed ||= over;
  console.log(
    `${over ? "OVER " : "ok   "}${kind.padEnd(4)} ${files.length} file(s)  raw ${kb(raw)} of ${kb(limit.raw)}  gzip ${kb(gzip)} of ${kb(limit.gzip)}`,
  );
}
if (failed) {
  console.error("Bundle budget exceeded. Shrink the bundle, or raise the limit in scripts/bundle-budget.mjs and say why.");
  process.exit(1);
}
