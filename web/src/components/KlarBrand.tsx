import { cn } from "cn";
import mark from "../../../docs/klar-mark.png";

export function KlarBrand({ className }: { className?: string }) {
  return (
    <span className={cn("inline-flex items-center gap-2", className)}>
      <img src={mark} alt="" className="h-9 w-9 shrink-0 object-contain" />
      <span className="brand">Klar!</span>
    </span>
  );
}
