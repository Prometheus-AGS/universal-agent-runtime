import { type FC, type ReactNode, useState } from "react";
import {
  AlertCircle,
  Check,
  ChevronDown,
  Eye,
  EyeOff,
  Loader2,
  RefreshCw,
  Save,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import { cn, friendlyError } from "@/lib/utils";

export const Field: FC<{
  label: string;
  hint?: string;
  defaultValue?: string;
  children: ReactNode;
}> = ({ label, hint, defaultValue, children }) => (
  <div className="space-y-1.5">
    <div className="flex items-baseline gap-2">
      <Label className="font-mono text-xs font-medium text-muted-foreground uppercase tracking-wide">
        {label}
      </Label>
      {defaultValue && (
        <span className="font-mono text-xs text-muted-foreground/50">
          default: {defaultValue}
        </span>
      )}
    </div>
    {children}
    {hint && (
      <p className="font-mono text-xs text-muted-foreground/60">{hint}</p>
    )}
  </div>
);

/** Collapsible "Advanced settings" section for progressive disclosure. */
export const AdvancedSection: FC<{
  label?: string;
  children: ReactNode;
  defaultOpen?: boolean;
}> = ({ label = "Advanced settings", children, defaultOpen = false }) => {
  const [open, setOpen] = useState(defaultOpen);
  return (
    <Collapsible open={open} onOpenChange={setOpen}>
      <CollapsibleTrigger
        render={
          <Button
            type="button"
            variant="ghost"
            className="h-auto w-full justify-start gap-2 rounded-lg border border-border/50 bg-muted/30 px-3 py-2 text-left font-normal hover:bg-muted/50"
          />
        }
      >
        <ChevronDown
          size={13}
          className={cn(
            "shrink-0 text-muted-foreground transition-transform duration-200",
            !open && "-rotate-90",
          )}
        />
        <span className="font-mono text-xs font-medium text-muted-foreground">
          {label}
        </span>
      </CollapsibleTrigger>
      <CollapsibleContent>
        <div className="mt-4 space-y-6 border-l-2 border-border/30 pl-4">
          {children}
        </div>
      </CollapsibleContent>
    </Collapsible>
  );
};

export const Toggle: FC<{
  value: boolean;
  onChange: (v: boolean) => void;
  disabled?: boolean;
}> = ({ value, onChange, disabled }) => (
  <Switch checked={value} onCheckedChange={onChange} disabled={disabled} />
);

export const MaskedInput: FC<{
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
}> = ({ value, onChange, placeholder }) => {
  const [show, setShow] = useState(false);
  return (
    <div className="flex gap-1.5">
      <Input
        type={show ? "text" : "password"}
        value={value}
        onChange={(e) => onChange(e.target.value)}
        placeholder={placeholder ?? "••••••••"}
        className="font-mono text-xs"
      />
      <Button
        variant="ghost"
        size="icon"
        className="shrink-0"
        onClick={() => setShow((s) => !s)}
        type="button"
        aria-label={show ? "Hide value" : "Show value"}
      >
        {show ? <EyeOff size={14} /> : <Eye size={14} />}
      </Button>
    </div>
  );
};

export const SettingSelect: FC<{
  value: string;
  options: { value: string; label: string }[];
  onChange: (v: string) => void;
}> = ({ value, options, onChange }) => (
  <Select value={value ?? ""} onValueChange={(v) => v != null && onChange(v)}>
    <SelectTrigger className="font-mono text-xs">
      <SelectValue />
    </SelectTrigger>
    <SelectContent>
      {options.map((o) => (
        <SelectItem key={o.value} value={o.value} className="font-mono text-xs">
          {o.label}
        </SelectItem>
      ))}
    </SelectContent>
  </Select>
);

export function PanelHeader({
  title,
  subtitle,
  saving,
  onSave,
  onReload,
  loading,
  saveDisabled = false,
}: {
  title: string;
  subtitle?: string;
  saving: boolean;
  loading: boolean;
  onSave: () => void;
  onReload: () => void;
  saveDisabled?: boolean;
}) {
  return (
    <div className="flex items-center justify-between border-b border-border bg-card px-6 py-4">
      <div>
        <h2 className="font-display text-lg font-semibold text-foreground">
          {title}
        </h2>
        {subtitle && (
          <p className="font-mono text-xs text-muted-foreground">{subtitle}</p>
        )}
      </div>
      <div className="flex items-center gap-2">
        <Button
          variant="outline"
          size="sm"
          onClick={onReload}
          disabled={loading}
          className="gap-1.5"
        >
          <RefreshCw size={13} className={cn(loading && "animate-spin")} />
          Refresh
        </Button>
        <Button
          size="sm"
          onClick={onSave}
          disabled={saving || saveDisabled}
          className="gap-1.5"
        >
          {saving ? (
            <Loader2 size={13} className="animate-spin" />
          ) : (
            <Save size={13} />
          )}
          Save
        </Button>
      </div>
    </div>
  );
}

export function SavedBanner({ show }: { show: boolean }) {
  if (!show) return null;
  return (
    <div className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-success/40 bg-success/10 px-4 py-2">
      <Check size={13} className="text-success" />
      <span className="font-mono text-xs text-success">Settings saved</span>
    </div>
  );
}

export function ErrorBanner({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <div className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-2">
      <AlertCircle size={13} className="text-destructive" />
      <span className="font-mono text-xs text-destructive">
        {friendlyError(error)}
      </span>
    </div>
  );
}

