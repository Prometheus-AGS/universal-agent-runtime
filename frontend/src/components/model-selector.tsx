import { useState, useMemo } from "react";
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
import { useModelSelector } from "@/hooks/use-model-selector";

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
  const { items, loading, error } = useModelSelector();

  /** Flatten catalog into grouped models, filtering to configured providers only. */
  const grouped = useMemo(() => {
    const groups = new Map<string, FlatModel[]>();
    for (const model of items) {
      if (model.provider_configured !== true) continue;
      const providerName = String(model.provider_name ?? model.provider_id);
      const modelId = String(model.model_id ?? model.id.split("/").slice(1).join("/"));
      const option: FlatModel = {
        value: model.id,
        name: model.name,
        id: modelId,
        providerName,
        providerId: model.provider_id,
        context: model.context,
      };
      groups.set(providerName, [...(groups.get(providerName) ?? []), option]);
    }
    for (const models of groups.values()) models.sort((a, b) => a.name.localeCompare(b.name));
    return groups;
  }, [items]);

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
      <PopoverTrigger
        render={
          <Button
            variant="outline"
            role="combobox"
            aria-expanded={open}
            className={cn(
              "h-9 w-full justify-between font-mono text-xs",
              !selectedLabel && !defaultLabel && "text-muted-foreground",
              className,
            )}
          />
        }
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
