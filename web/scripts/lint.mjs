// ESLint with a baseline. Violations that existed when lint was introduced are counted per file
// and rule in eslint-baseline.json; everything else fails. The baseline may only shrink:
//
//   npm run lint                        fail on any violation beyond the baseline, and on a
//                                       baseline entry that is higher than what is left
//   npm run lint -- --update-baseline   lower the baseline after fixing violations
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ESLint } from "eslint";

const webDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const baselineFile = path.join(webDir, "eslint-baseline.json");
const update = process.argv.includes("--update-baseline");

const baseline = fs.existsSync(baselineFile) ? JSON.parse(fs.readFileSync(baselineFile, "utf8")) : {};
const results = await new ESLint({ cwd: webDir }).lintFiles(["."]);

/** @type {Record<string, Record<string, { line: number, message: string }[]>>} */
const found = {};
const fatal = [];
for (const result of results) {
  const file = path.relative(webDir, result.filePath).split(path.sep).join("/");
  for (const message of result.messages) {
    if (message.fatal || !message.ruleId) {
      fatal.push(`${file}:${message.line}  ${message.message}`);
      continue;
    }
    ((found[file] ??= {})[message.ruleId] ??= []).push({ line: message.line, message: message.message.split("\n")[0] });
  }
}

const count = (file, rule) => found[file]?.[rule]?.length ?? 0;
const allowed = (file, rule) => baseline[file]?.[rule] ?? 0;

const grown = [];
for (const [file, rules] of Object.entries(found)) {
  for (const [rule, hits] of Object.entries(rules)) {
    if (hits.length <= allowed(file, rule)) continue;
    const over = hits.length - allowed(file, rule);
    grown.push(`${file}  ${rule}: ${hits.length} found, ${allowed(file, rule)} in the baseline (${over} new)`);
    for (const hit of hits) grown.push(`    ${file}:${hit.line}  ${hit.message}`);
  }
}
const stale = [];
for (const [file, rules] of Object.entries(baseline)) {
  for (const [rule, limit] of Object.entries(rules)) {
    if (count(file, rule) < limit) stale.push(`${file}  ${rule}: baseline ${limit}, found ${count(file, rule)}`);
  }
}

if (fatal.length) {
  console.error(`ESLint could not parse ${fatal.length} place(s):\n${fatal.join("\n")}`);
  process.exit(1);
}
if (grown.length) {
  console.error(`New lint violations (fix them; the baseline only shrinks):\n${grown.join("\n")}`);
  process.exit(1);
}
if (update) {
  const next = {};
  for (const file of Object.keys(found).sort()) {
    next[file] = {};
    for (const rule of Object.keys(found[file]).sort()) next[file][rule] = found[file][rule].length;
  }
  fs.writeFileSync(baselineFile, `${JSON.stringify(next, null, 2)}\n`);
  console.log(`eslint-baseline.json updated (${stale.length} entries lowered).`);
  process.exit(0);
}
if (stale.length) {
  console.error(`The lint baseline is higher than what is left. Run "npm run lint -- --update-baseline" and commit it:\n${stale.join("\n")}`);
  process.exit(1);
}
const total = Object.values(baseline).flatMap((rules) => Object.values(rules)).reduce((sum, n) => sum + n, 0);
console.log(`ESLint: no new violations (${total} known ones in eslint-baseline.json).`);
