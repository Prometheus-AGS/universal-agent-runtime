import { useState, useEffect, useMemo } from "react";
import { Check, ChevronsUpDown, X } from "lucide-react";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { fetchModelsCatalog } from "@/services/models-api";
import type { CatalogModelsResponse, CatalogProvider } from "@/types";

interface ModelSelectorProps {
  value: string;
  onChange: (value: string) => void;
  defaultLabel?: string;
  placeholder?: string;
  className?: string;
}

interface FlatModel {
  /** "provider/model-id" */
  value: string;
  /** Human-readable model name */
  name: string;
  /** Raw model id */
  id: string;
  /** Provider display name */
  providerName: string;
  /** Provider id */
  providerId: string;
  /** Context window size */
  context: number;
}

function formatContext(tokens: number): string {
  if (tokens >= 1_000_000) return `${Math.round(tokens / 1_000_000)}M`;
  if (tokens >= 1_000) return `${Math.round(tokens / 1_000)}k`;
  return String(tokens);
}

export function ModelSelector({
  value,
  onChange,
  defaultLabel,
  placeholder = "Select model...",
  className,
}: ModelSelectorProps) {
  const [open, setOpen] = useState(false);
  const [catalog, setCatalog] = useState<CatalogModelsResponse | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetchModelsCatalog()
      .then((data) => {
        if (!cancelled) setCatalog(data);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError((e as Error).message);
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  /** Flatten catalog into grouped models, filtering to configured providers only. */
  const grouped = useMemo(() => {
    if (!catalog) return new Map<string, FlatModel[]>();

    const groups = new Map<string, FlatModel[]>();

    const entries = Object.entries(catalog) as [string, CatalogProvider][];
    // Sort providers alphabetically by display name
    entries.sort(([, a], [, b]) =>
      a.display_name.localeCompare(b.display_name),
    );

    for (const [providerId, provider] of entries) {
      if (!provider.configured) continue;
      if (!provider.models || Object.keys(provider.models).length === 0)
        continue;

      const models: FlatModel[] = Object.entries(provider.models)
        .map(([modelId, model]) => ({
          value: `${providerId}/${modelId}`,
          name: model.name,
          id: modelId,
          providerName: provider.display_name,
          providerId,
          context: model.limit.context,
        }))
        .sort((a, b) => a.name.localeCompare(b.name));

      groups.set(provider.display_name, models);
    }

    return groups;
  }, [catalog]);

  /** Find the currently selected model's display name. */
  const selectedLabel = useMemo(() => {
    if (!value) return null;
    for (const models of grouped.values()) {
      const found = models.find((m) => m.value === value);
      if (found) return found.name;
    }
    // Fallback: show the raw value
    return value;
  }, [value, grouped]);

  const triggerLabel = selectedLabel ?? defaultLabel ?? placeholder;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          role="combobox"
          aria-expanded={open}
          className={cn(
            "h-9 w-full justify-between font-mono text-xs",
            !selectedLabel && !defaultLabel && "text-muted-foreground",
            className,
          )}
        >
          <span className="truncate">{triggerLabel}</span>
          <div className="flex items-center gap-1 shrink-0">
            {value && (
              <span
                role="button"
                tabIndex={0}
                className="rounded-sm p-0.5 hover:bg-accent"
                onClick={(e) => {
                  e.stopPropagation();
                  onChange("");
                }}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.stopPropagation();
                    onChange("");
                  }
                }}
              >
                <X className="h-3 w-3 text-muted-foreground" />
              </span>
            )}
            <ChevronsUpDown className="h-3.5 w-3.5 text-muted-foreground" />
          </div>
        </Button>
      </PopoverTrigger>
      <PopoverContent className="w-[400px] p-0" align="start">
        <Command>
          <CommandInput
            placeholder="Search models..."
            className="font-mono text-xs"
          />
          <CommandList>
            {loading && (
              <div className="py-6 text-center font-mono text-xs text-muted-foreground">
                Loading models...
              </div>
            )}
            {error && (
              <div className="py-6 text-center font-mono text-xs text-destructive">
                {error}
              </div>
            )}
            <CommandEmpty>No models found.</CommandEmpty>
            {[...grouped.entries()].map(([providerName, models]) => (
              <CommandGroup
                key={providerName}
                heading={
                  <span className="font-mono text-xs font-medium uppercase tracking-widest text-muted-foreground">
                    {providerName}
                  </span>
                }
              >
                {models.map((model) => (
                  <CommandItem
                    key={model.value}
                    value={`${model.providerName} ${model.name} ${model.id}`}
                    onSelect={() => {
                      onChange(model.value === value ? "" : model.value);
                      setOpen(false);
                    }}
                    className="font-mono text-xs"
                  >
                    <Check
                      className={cn(
                        "mr-2 h-3.5 w-3.5 shrink-0",
                        value === model.value
                          ? "opacity-100"
                          : "opacity-0",
                      )}
                    />
                    <span className="truncate">
                      {model.name}
                      {model.id !== model.name && (
                        <span className="ml-1.5 text-muted-foreground">
                          {model.id}
                        </span>
                      )}
                    </span>
                    <span className="ml-auto shrink-0 rounded bg-muted px-1.5 py-0.5 text-[10px] text-muted-foreground">
                      {formatContext(model.context)}
                    </span>
                  </CommandItem>
                ))}
              </CommandGroup>
            ))}
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
