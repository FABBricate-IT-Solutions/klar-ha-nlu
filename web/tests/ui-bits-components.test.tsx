import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { HouseLane } from "../src/components/HouseLane";
import { MatchLane } from "../src/components/MatchLane";
import { en } from "../src/i18n/en";

describe("HouseLane destructive delete (U-26)", () => {
  it("asks before removing a house rule", async () => {
    const user = userEvent.setup();
    const onRemove = vi.fn();
    render(
      <HouseLane
        t={en}
        rules={[{ id: "rule-ac", enabled: true, label: "No AC", when: { domain: "climate" }, effect: "confirm" }]}
        seedIds={new Set()}
        selected={0}
        bank={{ entries: [] }}
        intentOptions={[]}
        entityOptions={[]}
        rooms={[]}
        domains={[]}
        floors={[]}
        onSelect={() => undefined}
        onMove={() => undefined}
        onRemove={onRemove}
        onUpdate={() => undefined}
        onUpdateWhen={() => undefined}
        onUpdateWhenEntity={() => undefined}
        onBake={() => undefined}
      />,
    );
    await user.click(screen.getByRole("button", { name: en.dismiss }));
    expect(onRemove).not.toHaveBeenCalled();
    const dialog = await screen.findByRole("alertdialog");
    await user.click(within(dialog).getByRole("button", { name: en.dismiss }));
    expect(onRemove).toHaveBeenCalledWith("rule-ac");
  });

  it("exposes keyboard reorder controls", () => {
    render(
      <HouseLane
        t={en}
        rules={[
          { id: "a", enabled: true, label: "A", when: { domain: "light" }, effect: "allow" },
          { id: "b", enabled: true, label: "B", when: { domain: "switch" }, effect: "allow" },
        ]}
        seedIds={new Set()}
        selected={0}
        bank={{ entries: [] }}
        intentOptions={[]}
        entityOptions={[]}
        rooms={[]}
        domains={[]}
        floors={[]}
        onSelect={() => undefined}
        onMove={() => undefined}
        onRemove={() => undefined}
        onUpdate={() => undefined}
        onUpdateWhen={() => undefined}
        onUpdateWhenEntity={() => undefined}
        onBake={() => undefined}
      />,
    );
    expect(screen.getAllByRole("button", { name: en.moveRuleUp }).length).toBeGreaterThan(0);
    expect(screen.getAllByRole("button", { name: en.moveRuleDown }).length).toBeGreaterThan(0);
  });
});

describe("MatchLane precedence editing (U-24)", () => {
  it("lets the operator clear the precedence field without concatenating digits", async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(
      <MatchLane
        t={en}
        catalog={[{ id: "timer", precedence: 0, summary_key: "timer" }]}
        controls={[]}
        selected={0}
        onSelect={() => undefined}
        onChange={onChange}
      />,
    );
    const field = screen.getByLabelText(en.matchPrecedence);
    await user.clear(field);
    expect(field).toHaveValue(null);
    await user.type(field, "7");
    expect(onChange).toHaveBeenCalled();
    const last = onChange.mock.calls.at(-1)?.[0];
    expect(last).toEqual([{ id: "timer", enabled: true, precedence: 7 }]);
  });
});

describe("HouseLane labelled fields (U-25)", () => {
  it("labels the when fields instead of English-only placeholders", () => {
    render(
      <HouseLane
        t={en}
        rules={[{ id: "rule-ac", enabled: true, label: "No AC", when: { domain: "climate" }, effect: "confirm" }]}
        seedIds={new Set()}
        selected={0}
        bank={{ entries: [] }}
        intentOptions={[]}
        entityOptions={[]}
        rooms={[]}
        domains={[{ value: "climate", label: "climate" }]}
        floors={[]}
        onSelect={() => undefined}
        onMove={() => undefined}
        onRemove={() => undefined}
        onUpdate={() => undefined}
        onUpdateWhen={() => undefined}
        onUpdateWhenEntity={() => undefined}
        onBake={() => undefined}
      />,
    );
    expect(screen.getByLabelText(en.whenPhrase)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenIntent)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenDomain)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenArea)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenEntity)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenFloor)).toBeInTheDocument();
    expect(screen.getByLabelText(en.whenName)).toBeInTheDocument();
  });
});
