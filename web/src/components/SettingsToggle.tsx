import { FieldDescription, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import { cn } from "cn";

export function SettingsToggle({
  id,
  label,
  description,
  checked,
  onCheckedChange,
  className,
  disabled,
}: {
  id: string;
  label: string;
  description?: string;
  checked: boolean;
  onCheckedChange: (checked: boolean) => void;
  className?: string;
  disabled?: boolean;
}) {
  const hintId = description ? `${id}-hint` : undefined;
  return (
    <div
      role="group"
      className={cn("grid w-full grid-cols-[minmax(0,1fr)_auto] items-center gap-x-4 gap-y-1", className)}
    >
      <FieldLabel htmlFor={id} className="min-w-0">
        {label}
      </FieldLabel>
      <Switch
        id={id}
        className="col-start-2 row-start-1 shrink-0"
        checked={checked}
        disabled={disabled}
        aria-describedby={hintId}
        onCheckedChange={(next) => onCheckedChange(Boolean(next))}
      />
      {description ? (
        <FieldDescription id={hintId} className="col-start-1">
          {description}
        </FieldDescription>
      ) : null}
    </div>
  );
}
