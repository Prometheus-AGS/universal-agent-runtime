import { Fragment, type ReactNode } from "react";

import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Separator } from "@/components/ui/separator";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";
import type { A2uiComponent } from "@/features/a2ui/a2ui-protocol";

interface A2uiSurfaceRendererProps {
  components: A2uiComponent[];
  data: Record<string, unknown>;
  onDataChange: (path: string, value: unknown) => void;
  onAction: (name: string, context: Record<string, unknown>) => void;
  actionPending?: boolean;
  statusMessage?: string | null;
}

function resolvePath(data: Record<string, unknown>, path: string): unknown {
  if (!path.startsWith("/")) return undefined;
  return path.slice(1).split("/").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object") return undefined;
    const key = part.replaceAll("~1", "/").replaceAll("~0", "~");
    return (current as Record<string, unknown>)[key];
  }, data);
}

function dynamicValue(value: unknown, data: Record<string, unknown>): unknown {
  if (value && typeof value === "object" && "path" in value) {
    return resolvePath(data, String((value as { path: unknown }).path));
  }
  return value;
}

function bindingPath(value: unknown): string | null {
  return value && typeof value === "object" && "path" in value
    ? String((value as { path: unknown }).path)
    : null;
}

/** Pure projection of a validated A2UI surface onto UAR-owned React controls. */
export function A2uiSurfaceRenderer({
  components,
  data,
  onDataChange,
  onAction,
  actionPending = false,
  statusMessage,
}: A2uiSurfaceRendererProps) {
  const componentMap = new Map(components.map((component) => [component.id, component]));
  const root = componentMap.get("root") ?? components[0];

  const renderComponent = (componentId: string, ancestors: Set<string>): ReactNode => {
    if (ancestors.has(componentId)) {
      return <p className="text-sm text-destructive">A2UI component cycle detected at {componentId}.</p>;
    }
    const component = componentMap.get(componentId);
    if (!component) {
      return <p className="text-sm text-destructive">A2UI component {componentId} is unavailable.</p>;
    }
    const nextAncestors = new Set(ancestors).add(componentId);

    switch (component.component) {
      case "Text": {
        const text = String(dynamicValue(component.text, data) ?? "");
        if (component.variant === "h1") return <h2 className="text-lg font-semibold text-foreground">{text}</h2>;
        if (component.variant === "h2" || component.variant === "h3") {
          return <h3 className="text-base font-semibold text-foreground">{text}</h3>;
        }
        return <p className={cn("text-sm text-foreground", component.variant === "caption" && "text-xs text-muted-foreground")}>{text}</p>;
      }
      case "Button":
        {
          const child = componentMap.get(component.child);
          const label = child?.component === "Text"
            ? String(dynamicValue(child.text, data) ?? "")
            : renderComponent(component.child, nextAncestors);
        return (
          <Button
            type="button"
            size="sm"
            variant={component.variant === "borderless" ? "ghost" : component.variant === "secondary" ? "outline" : "default"}
            disabled={actionPending}
            onClick={() => onAction(component.action.event.name, component.action.event.context ?? {})}
          >
            {label}
          </Button>
        );
        }
      case "TextField": {
        const path = bindingPath(component.value);
        const value = String(dynamicValue(component.value, data) ?? "");
        const field = component.variant === "longText" ? (
          <Textarea
            id={component.id}
            value={value}
            placeholder={component.placeholder}
            disabled={actionPending || !path}
            rows={5}
            onChange={(event) => path && onDataChange(path, event.target.value)}
          />
        ) : (
          <Input
            id={component.id}
            type={component.variant === "email" ? "email" : component.variant === "number" ? "number" : "text"}
            value={value}
            placeholder={component.placeholder}
            disabled={actionPending || !path}
            onChange={(event) => path && onDataChange(path, event.target.value)}
          />
        );
        return <div className="space-y-1.5"><Label htmlFor={component.id}>{component.label}</Label>{field}</div>;
      }
      case "CheckBox": {
        const path = bindingPath(component.value);
        return (
          <div className="flex min-h-11 items-center gap-2">
            <Checkbox
              id={component.id}
              checked={Boolean(dynamicValue(component.value, data))}
              disabled={actionPending || !path}
              onCheckedChange={(checked) => path && onDataChange(path, checked === true)}
            />
            <Label htmlFor={component.id}>{component.label}</Label>
          </div>
        );
      }
      case "ChoicePicker": {
        const path = bindingPath(component.value);
        const selected = dynamicValue(component.value, data);
        const values = Array.isArray(selected) ? selected.map(String) : [];
        if (component.variant === "mutuallyExclusive") {
          return (
            <fieldset className="space-y-2">
              <legend className="text-sm font-medium text-foreground">{component.label}</legend>
              <RadioGroup
                value={values[0] ?? ""}
                disabled={actionPending || !path}
                onValueChange={(value) => path && onDataChange(path, [value])}
              >
                {component.options.map((option) => (
                  <div key={option.value} className="flex min-h-11 items-center gap-2">
                    <RadioGroupItem id={`${component.id}-${option.value}`} value={option.value} />
                    <Label htmlFor={`${component.id}-${option.value}`}>{option.label}</Label>
                  </div>
                ))}
              </RadioGroup>
            </fieldset>
          );
        }
        return (
          <fieldset className="space-y-2">
            <legend className="text-sm font-medium text-foreground">{component.label}</legend>
            {component.options.map((option) => (
              <div key={option.value} className="flex min-h-11 items-center gap-2">
                <Checkbox
                  id={`${component.id}-${option.value}`}
                  checked={values.includes(option.value)}
                  disabled={actionPending || !path}
                  onCheckedChange={(checked) => {
                    if (!path) return;
                    onDataChange(path, checked ? [...values, option.value] : values.filter((value) => value !== option.value));
                  }}
                />
                <Label htmlFor={`${component.id}-${option.value}`}>{option.label}</Label>
              </div>
            ))}
          </fieldset>
        );
      }
      case "Row":
        return <div className="flex flex-wrap items-center gap-2">{component.children.map((id) => <Fragment key={id}>{renderComponent(id, nextAncestors)}</Fragment>)}</div>;
      case "Column":
        return <div className="flex flex-col gap-3">{component.children.map((id) => <Fragment key={id}>{renderComponent(id, nextAncestors)}</Fragment>)}</div>;
      case "Card":
        return <div className="rounded-lg border border-border bg-card p-4">{renderComponent(component.child, nextAncestors)}</div>;
      case "Divider":
        return <Separator orientation={component.axis === "vertical" ? "vertical" : "horizontal"} />;
    }
  };

  return (
    <div className="space-y-3" data-a2ui-surface>
      {root ? renderComponent(root.id, new Set()) : <p className="text-sm text-muted-foreground">This A2UI surface has no components yet.</p>}
      <div className="min-h-5 text-xs text-muted-foreground" role="status" aria-live="polite">
        {statusMessage}
      </div>
    </div>
  );
}
