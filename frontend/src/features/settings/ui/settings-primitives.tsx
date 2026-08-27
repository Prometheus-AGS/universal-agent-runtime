import { type FC, type ReactNode, useId, useState } from "react";
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
import {
  Combobox,
  ComboboxContent,
  ComboboxEmpty,
  ComboboxInput,
  ComboboxItem,
  ComboboxList,
  ComboboxTrigger,
} from "@/components/ui/combobox";
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
  htmlFor?: string;
  hintId?: string;
  children: ReactNode;
}> = ({ label, hint, defaultValue, htmlFor, hintId, children }) => (
  <div className="min-w-0 space-y-1.5">
    <div className="flex items-baseline gap-2">
      <Label
        htmlFor={htmlFor}
        className="font-mono text-xs font-medium text-muted-foreground uppercase tracking-wide"
      >
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
      <p id={hintId} className="font-mono text-xs text-muted-foreground/60">
        {hint}
      </p>
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
  ariaDisabled?: boolean;
  id?: string;
  ariaLabel?: string;
  ariaDescribedBy?: string;
}> = ({
  value,
  onChange,
  disabled,
  ariaDisabled,
  id,
  ariaLabel,
  ariaDescribedBy,
}) => (
  <Switch
    id={id}
    aria-label={ariaLabel}
    aria-describedby={ariaDescribedBy}
    aria-disabled={ariaDisabled || undefined}
    checked={value}
    onCheckedChange={(checked) => {
      if (!ariaDisabled) onChange(checked);
    }}
    onClick={(event) => {
      if (ariaDisabled) event.preventDefault();
    }}
    onKeyDown={(event) => {
      if (ariaDisabled && (event.key === " " || event.key === "Enter")) {
        event.preventDefault();
      }
    }}
    disabled={disabled}
    className="aria-disabled:cursor-not-allowed aria-disabled:opacity-50"
  />
);

export const MaskedInput: FC<{
  value: string;
  onChange: (v: string) => void;
  placeholder?: string;
  id?: string;
  ariaDescribedBy?: string;
  revealLabel?: string;
}> = ({
  value,
  onChange,
  placeholder,
  id,
  ariaDescribedBy,
  revealLabel = "value",
}) => {
  const [show, setShow] = useState(false);
  return (
    <div className="flex gap-1.5">
      <Input
        id={id}
        aria-describedby={ariaDescribedBy}
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
        aria-label={`${show ? "Hide" : "Show"} ${revealLabel}`}
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
  triggerClassName?: string;
  ariaLabel?: string;
  disabled?: boolean;
  placeholder?: string;
  ariaInvalid?: boolean;
  id?: string;
  ariaDescribedBy?: string;
}> = ({
  value,
  options,
  onChange,
  triggerClassName,
  ariaLabel,
  disabled,
  placeholder,
  ariaInvalid,
  id,
  ariaDescribedBy,
}) => (
  <Select
    items={options}
    value={value ?? ""}
    onValueChange={(v) => v != null && onChange(v)}
    disabled={disabled}
  >
    <SelectTrigger
      id={id}
      className={cn("font-mono text-xs", triggerClassName)}
      aria-label={ariaLabel}
      aria-invalid={ariaInvalid || undefined}
      aria-describedby={ariaDescribedBy}
    >
      <SelectValue placeholder={placeholder} />
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

type SettingOption = { value: string; label: string };

export const SEARCHABLE_MODEL_THRESHOLD = 8;

function normalizeModelSearch(value: string) {
  return value.trim().toLowerCase();
}

export const SettingModelPicker: FC<{
  value: string;
  options: SettingOption[];
  onChange: (v: string) => void;
  triggerClassName?: string;
  ariaLabel: string;
  searchAriaLabel: string;
  disabled?: boolean;
  placeholder?: string;
  ariaInvalid?: boolean;
  id?: string;
  ariaDescribedBy?: string;
}> = ({
  value,
  options,
  onChange,
  triggerClassName,
  ariaLabel,
  searchAriaLabel,
  disabled,
  placeholder = "Select a model",
  ariaInvalid,
  id,
  ariaDescribedBy,
}) => {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState("");
  const selectedOption =
    options.find((option) => option.value === value) ?? null;

  if (options.length < SEARCHABLE_MODEL_THRESHOLD) {
    return (
      <SettingSelect
        value={value}
        options={options}
        onChange={onChange}
        triggerClassName={triggerClassName}
        ariaLabel={ariaLabel}
        disabled={disabled}
        placeholder={placeholder}
        ariaInvalid={ariaInvalid}
        id={id}
        ariaDescribedBy={ariaDescribedBy}
      />
    );
  }

  return (
    <Combobox
      items={options}
      value={selectedOption}
      open={open}
      inputValue={query}
      onInputValueChange={setQuery}
      onOpenChange={(nextOpen) => {
        setOpen(nextOpen);
        if (!nextOpen) setQuery("");
      }}
      onValueChange={(option) => {
        if (option && option.value !== value) onChange(option.value);
        setOpen(false);
        setQuery("");
      }}
      filter={(option, inputValue) => {
        const needle = normalizeModelSearch(inputValue);
        if (!needle) return true;
        return (
          normalizeModelSearch(option.label).includes(needle) ||
          normalizeModelSearch(option.value).includes(needle)
        );
      }}
      itemToStringLabel={(option) => option.label}
      itemToStringValue={(option) => option.value}
      isItemEqualToValue={(option, selected) => option.value === selected.value}
      autoHighlight
      disabled={disabled}
    >
      <ComboboxTrigger
        id={id}
        aria-label={ariaLabel}
        aria-invalid={ariaInvalid || undefined}
        aria-describedby={ariaDescribedBy}
        disabled={disabled}
        className={cn(
          "flex h-9 w-full items-center justify-between gap-1.5 rounded-md border border-input bg-transparent py-2 pr-2 pl-2.5 font-mono text-xs whitespace-nowrap transition-[color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-3 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 aria-invalid:border-destructive aria-invalid:ring-3 aria-invalid:ring-destructive/20 dark:bg-input/30 dark:hover:bg-input/50 dark:aria-invalid:border-destructive/50 dark:aria-invalid:ring-destructive/40",
          triggerClassName,
        )}
      >
        <span
          className={cn(
            "min-w-0 flex-1 truncate text-left",
            !selectedOption && "text-muted-foreground",
          )}
        >
          {selectedOption?.label ?? placeholder}
        </span>
      </ComboboxTrigger>
      <ComboboxContent className="min-w-(--anchor-width)">
        <ComboboxInput
          autoFocus
          showTrigger={false}
          aria-label={searchAriaLabel}
          placeholder="Search models…"
          className="w-full font-mono text-xs"
        />
        <ComboboxEmpty className="font-mono text-xs">
          No matching models.
        </ComboboxEmpty>
        <ComboboxList>
          {(option: SettingOption) => (
            <ComboboxItem
              key={option.value}
              value={option}
              className="items-start font-mono text-xs"
            >
              <span className="min-w-0 flex-1 break-words">{option.label}</span>
              {option.value !== option.label && (
                <span className="min-w-0 break-all text-[10px] text-muted-foreground">
                  {option.value}
                </span>
              )}
            </ComboboxItem>
          )}
        </ComboboxList>
      </ComboboxContent>
    </Combobox>
  );
};

export function PanelHeader({
  title,
  subtitle,
  saving,
  onSave,
  onReload,
  loading,
  saveDisabled = false,
  reloadDisabled = false,
  statusText,
  reloadHint,
}: {
  title: string;
  subtitle?: string;
  saving: boolean;
  loading: boolean;
  onSave: () => void;
  onReload: () => void;
  saveDisabled?: boolean;
  reloadDisabled?: boolean;
  statusText?: string;
  reloadHint?: string;
}) {
  const reloadHintId = useId();
  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-card px-6 py-4">
      <div className="min-w-0">
        <h2 className="font-display text-lg font-semibold text-foreground">
          {title}
        </h2>
        {subtitle && (
          <p className="font-mono text-xs text-muted-foreground">{subtitle}</p>
        )}
        {statusText && (
          <p
            role="status"
            aria-live="polite"
            className="font-mono text-xs font-medium text-warning"
          >
            {statusText}
          </p>
        )}
      </div>
      <div className="flex min-w-0 flex-col items-end gap-1">
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={onReload}
            disabled={loading || reloadDisabled}
            aria-describedby={reloadHint ? reloadHintId : undefined}
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
        {reloadHint && (
          <p
            id={reloadHintId}
            className="font-mono text-[10px] text-muted-foreground"
          >
            {reloadHint}
          </p>
        )}
      </div>
    </div>
  );
}

export function SavedBanner({ show }: { show: boolean }) {
  if (!show) return null;
  return (
    <div
      role="status"
      aria-live="polite"
      aria-atomic="true"
      className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-success/40 bg-success/10 px-4 py-2"
    >
      <Check size={13} className="text-success" />
      <span className="font-mono text-xs text-success">Settings saved</span>
    </div>
  );
}

export function ErrorBanner({ error }: { error: string | null }) {
  if (!error) return null;
  return (
    <div
      role="alert"
      aria-atomic="true"
      className="mx-6 mb-4 flex items-center gap-2 rounded-lg border border-destructive/40 bg-destructive/10 px-4 py-2"
    >
      <AlertCircle size={13} className="text-destructive" />
      <span className="font-mono text-xs text-destructive">
        {friendlyError(error)}
      </span>
    </div>
  );
}
