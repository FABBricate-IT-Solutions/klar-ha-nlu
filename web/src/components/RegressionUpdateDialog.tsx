import { useEffect, useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { api } from "../api";
import type { Messages } from "../i18n";
import type { RegressionChangedCase } from "../types";

type Pending = { changed: number; cases: RegressionChangedCase[] };

/** After an engine update, lists saved per-home cases whose behaviour moved (package 6.3). */
export function RegressionUpdateDialog({ active, t }: { active: boolean; t: Messages }) {
  const [pending, setPending] = useState<Pending | null>(null);

  useEffect(() => {
    if (!active) return;
    let live = true;
    api.wrongCaseUpdateDialog()
      .then((dialog) => {
        if (!live) return;
        if (dialog.is_update && dialog.changed > 0) {
          setPending({ changed: dialog.changed, cases: dialog.cases ?? [] });
        }
      })
      .catch(() => undefined);
    return () => {
      live = false;
    };
  }, [active]);

  const sentences = (pending?.cases ?? []).map((row) => row.text).filter(Boolean);
  return (
    <AlertDialog
      open={pending !== null}
      onOpenChange={(next) => {
        if (!next) setPending(null);
      }}
    >
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t.regressionUpdateTitle}</AlertDialogTitle>
          <AlertDialogDescription>
            {t.regressionUpdateCount.replace("{n}", String(pending?.changed ?? 0))}
          </AlertDialogDescription>
        </AlertDialogHeader>
        {sentences.length > 0 ? (
          <ul className="regression-sentences mono">
            {sentences.map((text) => (
              <li key={text}>{text}</li>
            ))}
          </ul>
        ) : null}
        <AlertDialogFooter>
          <AlertDialogCancel onClick={() => setPending(null)}>{t.cancel}</AlertDialogCancel>
          <AlertDialogAction
            onClick={() => {
              void api.ackWrongCaseUpdateDialog().finally(() => setPending(null));
            }}
          >
            {t.regressionUpdateAck}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
