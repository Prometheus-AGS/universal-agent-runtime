import {
  type ReactNode,
  useCallback,
  useEffect,
  useMemo,
  useState,
} from "react";
import { AlertCircle, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { cn } from "@/lib/utils";
import { useOnboarding } from "../model/use-onboarding";
import { useSettings } from "../model/use-settings";
import { useSettingsTypesMeta } from "../model/use-settings-types-meta";
import {
  ErrorBanner,
  MaskedInput,
  PanelHeader,
  SavedBanner,
  SettingSelect,
  Toggle,
} from "./settings-primitives";
import { parseNumberInput, toStringValue } from "./settings-value-utils";

export function SettingsHint({ id, children }: { id: string; children: ReactNode }) {
  const { dismissed, dismiss } = useOnboarding(`settings-hint-${id}`);
  if (dismissed) return null;
  return (
    <div className="mb-4 flex items-start gap-2.5 rounded-lg border border-primary/20 bg-primary/5 px-3 py-2.5">
      <AlertCircle size={13} className="mt-0.5 shrink-0 text-primary" />
      <p className="flex-1 font-body text-xs leading-relaxed text-muted-foreground">
        {children}
      </p>
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={dismiss}
        className="h-auto shrink-0 px-1 py-0.5 text-muted-foreground hover:text-foreground"
        aria-label="Dismiss hint"
      >
        <span className="text-xs">Got it</span>
      </Button>
    </div>
  );
}

export function NamespacePanel({
  namespace,
  title,
  subtitle,
  hint,
  saveDisabled,
  children,
}: {
  namespace: string;
  title: string;
  subtitle?: string;
  hint?: string;
  saveDisabled?: boolean;
  children: (ctx: {
    val: (key: string) => unknown;
    set: (key: string, value: unknown) => void;
  }) => ReactNode;
}) {
  const { values, loading, saving, error, setSetting, saveAll, reload } =
    useSettings(namespace);
  const [savedFlash, setSavedFlash] = useState(false);

  const handleSave = useCallback(async () => {
    await saveAll();
    setSavedFlash(true);
    setTimeout(() => setSavedFlash(false), 2500);
  }, [saveAll]);

  const val = (key: string) => values[`${namespace}.${key}`];
  const set = (key: string, value: unknown) =>
    setSetting(`${namespace}.${key}`, value);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title={title}
        subtitle={subtitle}
        saving={saving}
        loading={loading}
        saveDisabled={saveDisabled}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading && (
          <div className="flex items-center gap-2">
            <Loader2 size={15} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">
              Loading…
            </span>
          </div>
        )}
        <ErrorBanner error={error} />
        <SavedBanner show={savedFlash} />
        {hint && <SettingsHint id={namespace}>{hint}</SettingsHint>}
        {!loading && <div className="space-y-6">{children({ val, set })}</div>}
      </div>
    </div>
  );
}

type SchemaProperty = {
  type?: string | string[];
  title?: string;
  description?: string;
  enum?: Array<string | number | boolean>;
  default?: unknown;
  properties?: Record<string, SchemaProperty>;
  items?: SchemaProperty;
  minimum?: number;
  maximum?: number;
  "x-control"?: string;
  "x-sensitive"?: boolean;
};

function schemaProperties(
  schema: Record<string, unknown>,
): Record<string, SchemaProperty> {
  const properties = schema.properties;
  return properties &&
    typeof properties === "object" &&
    !Array.isArray(properties)
    ? (properties as Record<string, SchemaProperty>)
    : {};
}

function leafKey(fullKey: string, namespace: string): string {
  return fullKey.startsWith(`${namespace}.`)
    ? fullKey.slice(namespace.length + 1)
    : fullKey;
}

function settingLabel(
  rowName: string,
  key: string,
  prop?: SchemaProperty,
): string {
  if (prop?.title) return prop.title;
  if (rowName && rowName !== key) return rowName;
  return key
    .split(/[._-]/)
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ");
}

function schemaType(prop: SchemaProperty | undefined, value: unknown): string {
  if (prop?.enum) return "enum";
  const explicit = Array.isArray(prop?.type) ? prop?.type[0] : prop?.type;
  if (explicit) return explicit;
  if (Array.isArray(value)) return "array";
  if (typeof value === "boolean") return "boolean";
  if (typeof value === "number") return "number";
  if (value && typeof value === "object") return "object";
  return "string";
}

function JsonSettingEditor({
  value,
  onChange,
}: {
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  const [draft, setDraft] = useState(() => toStringValue(value));
  const [parseError, setParseError] = useState<string | null>(null);

  useEffect(() => {
    queueMicrotask(() => {
      setDraft(toStringValue(value));
      setParseError(null);
    });
  }, [value]);

  return (
    <div className="space-y-1.5">
      <textarea
        value={draft}
        onChange={(event) => {
          const next = event.target.value;
          setDraft(next);
          try {
            onChange(next.trim() ? JSON.parse(next) : {});
            setParseError(null);
          } catch (error) {
            setParseError((error as Error).message);
          }
        }}
        rows={8}
        spellCheck={false}
        className="min-h-28 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-xs shadow-sm outline-none transition-colors focus-visible:ring-3 focus-visible:ring-ring"
      />
      {parseError && (
        <p className="font-mono text-xs text-destructive">
          JSON is not valid: {parseError}
        </p>
      )}
    </div>
  );
}

function GenericSettingControl({
  prop,
  value,
  onChange,
}: {
  prop?: SchemaProperty;
  value: unknown;
  onChange: (value: unknown) => void;
}) {
  const type = schemaType(prop, value);

  if (type === "boolean") {
    return <Toggle value={Boolean(value)} onChange={onChange} />;
  }

  if (type === "enum" && prop?.enum) {
    const options = prop.enum.map((item) => ({
      value: String(item),
      label: String(item),
    }));
    return (
      <SettingSelect
        value={String(value ?? prop.default ?? options[0]?.value ?? "")}
        options={options}
        onChange={onChange}
      />
    );
  }

  if (type === "integer" || type === "number") {
    return (
      <Input
        type="number"
        min={prop?.minimum}
        max={prop?.maximum}
        value={(value as number | null | undefined) ?? ""}
        onChange={(event) => onChange(parseNumberInput(event.target.value))}
        className="font-mono text-xs"
      />
    );
  }

  if (type === "array") {
    return (
      <Input
        value={toStringValue(value)}
        onChange={(event) => {
          const next = event.target.value
            .split(",")
            .map((item) => item.trim())
            .filter(Boolean);
          onChange(next);
        }}
        placeholder="value/one, value/two"
        className="font-mono text-xs"
      />
    );
  }

  if (type === "object" || prop?.["x-control"] === "json") {
    return <JsonSettingEditor value={value} onChange={onChange} />;
  }

  if (prop?.["x-sensitive"]) {
    return <MaskedInput value={toStringValue(value)} onChange={onChange} />;
  }

  if (prop?.["x-control"] === "textarea") {
    return (
      <textarea
        value={toStringValue(value)}
        onChange={(event) => onChange(event.target.value)}
        rows={4}
        className="min-h-20 w-full rounded-md border border-input bg-background px-3 py-2 font-mono text-xs shadow-sm outline-none transition-colors focus-visible:ring-3 focus-visible:ring-ring"
      />
    );
  }

  return (
    <Input
      value={toStringValue(value)}
      onChange={(event) => onChange(event.target.value)}
      className="font-mono text-xs"
    />
  );
}

export function GenericSchemaPanel({
  namespace,
  title,
  subtitle,
  hint,
}: {
  namespace: string;
  title: string;
  subtitle?: string;
  hint?: string;
}) {
  const {
    values,
    settings,
    dirty,
    conflicts,
    loading,
    saving,
    error,
    setSetting,
    saveAll,
    reload,
  } = useSettings(namespace);
  const types = useSettingsTypesMeta();
  const [savedFlash, setSavedFlash] = useState(false);
  const typeMeta = types.find((type) => type.key === namespace);
  const properties = useMemo(
    () => schemaProperties(typeMeta?.schema ?? {}),
    [typeMeta],
  );
  const rows = useMemo(
    () => Object.values(settings).sort((a, b) => a.key.localeCompare(b.key)),
    [settings],
  );

  const handleSave = useCallback(async () => {
    await saveAll();
    setSavedFlash(true);
    setTimeout(() => setSavedFlash(false), 2500);
  }, [saveAll]);

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PanelHeader
        title={title}
        subtitle={subtitle ?? `${rows.length} setting(s)`}
        saving={saving}
        loading={loading}
        saveDisabled={Object.keys(dirty).length === 0}
        onSave={() => void handleSave()}
        onReload={() => void reload()}
      />
      <div className="flex-1 overflow-y-auto px-6 py-5">
        {loading && (
          <div className="flex items-center gap-2">
            <Loader2 size={15} className="animate-spin text-muted-foreground" />
            <span className="font-mono text-xs text-muted-foreground">
              Loading…
            </span>
          </div>
        )}
        <ErrorBanner error={error} />
        <SavedBanner show={savedFlash} />
        {hint && <SettingsHint id={namespace}>{hint}</SettingsHint>}
        {!loading && rows.length === 0 && (
          <div className="rounded-lg border border-dashed border-border px-4 py-8 text-center">
            <p className="font-mono text-xs text-muted-foreground">
              No editable settings are registered for this namespace.
            </p>
          </div>
        )}
        {!loading && rows.length > 0 && (
          <div className="grid gap-4 lg:grid-cols-2">
            {rows.map((row) => {
              const key = leafKey(row.key, namespace);
              const prop = properties[key];
              const isDirty = Object.prototype.hasOwnProperty.call(
                dirty,
                row.key,
              );
              const conflict = conflicts[row.key];
              return (
                <div
                  key={row.key}
                  className={cn(
                    "space-y-3 rounded-lg border border-border bg-card px-4 py-3",
                    conflict !== undefined &&
                      "border-amber-500/50 bg-amber-500/5",
                  )}
                >
                  <div className="flex items-start justify-between gap-3">
                    <div className="min-w-0">
                      <p className="font-mono text-xs font-medium text-foreground">
                        {settingLabel(row.name, key, prop)}
                      </p>
                      <p className="mt-0.5 break-all font-mono text-xs text-muted-foreground/70">
                        {row.key}
                      </p>
                      {prop?.description && (
                        <p className="mt-1 font-mono text-xs leading-relaxed text-muted-foreground">
                          {prop.description}
                        </p>
                      )}
                    </div>
                    <div className="flex shrink-0 items-center gap-1.5">
                      {isDirty && (
                        <span className="rounded-full bg-primary/10 px-2 py-0.5 font-mono text-[10px] uppercase tracking-wide text-primary">
                          Modified
                        </span>
                      )}
                      {prop?.default !== undefined && (
                        <Button
                          type="button"
                          variant="ghost"
                          size="sm"
                          className="h-7 px-2 font-mono text-xs"
                          onClick={() => setSetting(row.key, prop.default)}
                        >
                          Reset
                        </Button>
                      )}
                    </div>
                  </div>
                  <GenericSettingControl
                    prop={prop}
                    value={values[row.key]}
                    onChange={(next) => setSetting(row.key, next)}
                  />
                  {conflict !== undefined && (
                    <p className="font-mono text-xs text-amber-600 dark:text-amber-400">
                      Remote update received while this field has local edits.
                      Reload to accept the remote value.
                    </p>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
}
