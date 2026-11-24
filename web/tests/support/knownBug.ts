import { it } from "vitest";

/**
 * A test for a defect that is not fixed yet. `body` asserts the correct behaviour. The test passes
 * only while `body` fails with a message matching `symptom`, so it documents the bug without
 * keeping CI red. Two things turn it red:
 *
 * - `body` passes: the bug is fixed. Replace `knownBug(...)` with a plain `it(...)`.
 * - `body` fails differently: the test no longer reaches the bug (a renamed label, a broken
 *   harness). Repair it, otherwise it proves nothing.
 */
export function knownBug(finding: string, title: string, symptom: RegExp, body: () => Promise<void>): void {
  it(`${title} [known bug ${finding}]`, async () => {
    let failure: unknown;
    let failed = false;
    try {
      await body();
    } catch (err) {
      failure = err;
      failed = true;
    }
    if (!failed) {
      throw new Error(`${finding} no longer reproduces: replace knownBug(...) with it(...) for "${title}".`);
    }
    const message = failure instanceof Error ? failure.message : String(failure);
    if (!symptom.test(message)) {
      throw new Error(`${finding}: "${title}" failed, but not with the known symptom ${String(symptom)}.\n${message}`, {
        cause: failure,
      });
    }
  });
}
