import type { ReactNode } from "react";

export function Snackbar({
  message,
  action,
  onDismiss,
  dismissLabel,
  tone = "default",
}: {
  message: string;
  action?: ReactNode;
  onDismiss?: () => void;
  dismissLabel: string;
  tone?: "default" | "danger";
}) {
  return (
    <div className={`snackbar${tone === "danger" ? " danger" : ""}`} role="status" aria-live="polite">
      <span>{message}</span>
      <div className="row">
        {action}
        {onDismiss && (
          <button className="ghost" type="button" onClick={onDismiss} aria-label={dismissLabel}>
            {dismissLabel}
          </button>
        )}
      </div>
    </div>
  );
}
