// Bundle budget for the built UI (run after `npm run build`). The limits sit just above what the
// bundle weighs today, so the check stops growth. Lower them when the bundle shrinks (plan
// package 2.17 splits the language packs out of the main chunk); raise one only with a reason in
// the pull request.
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { gzipSync } from "node:zlib";

const KB = 1024;
const BUDGET = {
  js: { raw: 2900 * KB, gzip: 790 * KB },
  css: { raw: 128 * KB, gzip: 22 * KB },
};

const assets = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "dist", "assets");
if (!fs.existsSync(assets)) {
  console.error("web/dist/assets is missing: run `npm run build` first.");
  process.exit(1);
}

const kb = (bytes) => `${(bytes / KB).toFixed(1)} kB`;
let failed = false;
for (const [kind, limit] of Object.entries(BUDGET)) {
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
