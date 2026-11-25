import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { ErrorBoundary } from "../src/components/ErrorBoundary";
import { saveFailMessage, toastActionFailed, toastSaveFailed } from "../src/saveToast";
import { ApiError } from "../src/lib/apiError";
import { en } from "../src/i18n/en";

vi.mock("sonner", () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}));

import { toast } from "sonner";

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("ErrorBoundary (U-19)", () => {
  it("shows a reload card instead of a blank root when a child throws", () => {
    function Boom(): null {
      throw new Error("URI malformed");
    }
    render(
      <ErrorBoundary>
        <Boom />
      </ErrorBoundary>,
    );
    expect(screen.getByRole("alert")).toHaveTextContent("Something broke on this page");
    expect(screen.getByRole("button", { name: "Reload" })).toBeInTheDocument();
  });
});

describe("action error toasts (U-18)", () => {
  it("prefers the engine error text over a generic save failure", () => {
    const err = new ApiError(500, "Internal Server Error", { error: "could not save: disk full", path: "klar_nlu.json" });
    expect(saveFailMessage(en, err)).toBe("could not save: disk full");
    toastSaveFailed(en, err);
    expect(toast.error).toHaveBeenCalledWith("could not save: disk full", { description: "klar_nlu.json" });
  });

  it("shows a 500 engine message for a non-save action", () => {
    toastActionFailed(new ApiError(500, "Internal Server Error", { error: "overlay locked", path: "policies" }));
    expect(toast.error).toHaveBeenCalledWith("overlay locked", { description: "policies" });
  });
});
