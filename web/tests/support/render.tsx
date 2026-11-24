import { render, type RenderResult } from "@testing-library/react";
import userEvent, { type UserEvent } from "@testing-library/user-event";
import type { ReactElement } from "react";
import { TooltipProvider } from "../../src/components/ui/tooltip";

export type User = UserEvent;

/** Renders a page the way `App` hosts it and returns a user that types and clicks like an operator. */
export function renderUi(ui: ReactElement): RenderResult & { user: User } {
  const user = userEvent.setup();
  return { user, ...render(<TooltipProvider>{ui}</TooltipProvider>) };
}

/** Waits out a timer in the UI (a debounce, a delayed save). To wait for requests, use `engine.idle()`. */
export async function settle(ms: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, ms));
}
