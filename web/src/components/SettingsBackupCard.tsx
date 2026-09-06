import { useRef, useState } from "react";
import { api, download, uploadSettingsBackup } from "../api";
import type { Messages } from "../i18n";
import type { Settings } from "../types";
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
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from "@/components/ui/card";
import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldContent, FieldDescription, FieldGroup, FieldLabel } from "@/components/ui/field";

export function SettingsBackupCard({
  t,
  onSettings,
  onRestored,
}: {
  t: Messages;
  onSettings: (settings: Settings) => void;
  onRestored?: () => void;
}) {
  const fileRef = useRef<HTMLInputElement>(null);
  const [includeKey, setIncludeKey] = useState(false);
  const [picked, setPicked] = useState<File | null>(null);
  const [confirmSecrets, setConfirmSecrets] = useState(false);
  const [confirmRestore, setConfirmRestore] = useState(false);
  const [status, setStatus] = useState("");

  const pull = async (secrets: boolean) => {
    const path = secrets ? "/api/v2/settings/backup?secrets=1" : "/api/v2/settings/backup";
    await download(path, "klar-settings.tar.gz");
    setStatus(t.settingsBackupDownload);
  };

  const startDownload = () => {
    if (includeKey) {
      setConfirmSecrets(true);
      return;
    }
    void pull(false).catch(() => setStatus(t.settingsBackupRestoreFail));
  };

  const restore = async () => {
    if (!picked) {
      return;
    }
    await uploadSettingsBackup(picked);
    const next = await api.settings();
    onSettings(next);
    onRestored?.();
    setPicked(null);
    if (fileRef.current) {
      fileRef.current.value = "";
    }
    setStatus(t.settingsBackupRestoreOk);
  };

  return (
    <>
      <Card className="md:col-span-2">
        <CardHeader>
          <CardTitle>{t.settingsBackup}</CardTitle>
          <CardDescription>{t.settingsBackupHint}</CardDescription>
        </CardHeader>
        <CardContent>
          <FieldGroup>
            <Field orientation="horizontal">
              <FieldContent>
                <FieldLabel htmlFor="klar-backup-secrets">{t.settingsBackupIncludeKey}</FieldLabel>
                <FieldDescription>{t.settingsBackupIncludeKeyHint}</FieldDescription>
              </FieldContent>
              <Checkbox
                id="klar-backup-secrets"
                checked={includeKey}
                onCheckedChange={(checked) => setIncludeKey(checked === true)}
              />
            </Field>
            <Field>
              <FieldLabel>{t.settingsBackupPickFile}</FieldLabel>
              <input
                ref={fileRef}
                className="sr-only"
                type="file"
                accept=".tar.gz,.tgz,.tar,.gz,application/gzip,application/x-tar"
                onChange={(ev) => setPicked(ev.target.files?.[0] || null)}
              />
              <Button variant="outline" type="button" onClick={() => fileRef.current?.click()}>
                {t.settingsBackupPickFile}
              </Button>
              {picked ? <FieldDescription>{picked.name}</FieldDescription> : null}
            </Field>
            {status ? <p className="caption">{status}</p> : null}
          </FieldGroup>
        </CardContent>
        <CardFooter className="flex flex-wrap gap-2">
          <Button variant="outline" type="button" onClick={startDownload}>
            {t.settingsBackupDownload}
          </Button>
          <Button variant="outline" type="button" disabled={!picked} onClick={() => setConfirmRestore(true)}>
            {t.settingsBackupRestore}
          </Button>
        </CardFooter>
      </Card>
      <AlertDialog open={confirmSecrets} onOpenChange={setConfirmSecrets}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t.settingsBackupIncludeKey}</AlertDialogTitle>
            <AlertDialogDescription>{t.settingsBackupIncludeKeyConfirm}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                void pull(true).catch(() => setStatus(t.settingsBackupRestoreFail));
              }}
            >
              {t.settingsBackupDownload}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <AlertDialog open={confirmRestore} onOpenChange={setConfirmRestore}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{t.settingsBackupRestore}</AlertDialogTitle>
            <AlertDialogDescription>{t.settingsBackupRestoreConfirm}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{t.cancel}</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                void restore().catch(() => setStatus(t.settingsBackupRestoreFail));
              }}
            >
              {t.settingsBackupRestore}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
