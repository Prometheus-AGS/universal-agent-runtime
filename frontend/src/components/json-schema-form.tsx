import { type FC, useCallback } from "react";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch } from "@/components/ui/switch";
import { Textarea } from "@/components/ui/textarea";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { cn } from "@/lib/utils";

interface JsonSchemaFormProps {
  schema: Record<string, unknown>;
  value: Record<string, unknown>;
  onChange: (value: Record<string, unknown>) => void;
  className?: string;
}

export const JsonSchemaForm: FC<JsonSchemaFormProps> = ({ schema, value, onChange, className }) => {
  const properties = (schema.properties ?? {}) as Record<string, Record<string, unknown>>;
  const required = (schema.required ?? []) as string[];

  if (Object.keys(properties).length === 0) {
    return (
      <p className="font-mono text-xs text-muted-foreground">No input parameters defined.</p>
    );
  }

  return (
    <div className={cn("flex flex-col gap-4", className)}>
      {Object.entries(properties).map(([key, propSchema]) => (
        <SchemaField
          key={key}
          name={key}
          schema={propSchema}
          value={value[key]}
          required={required.includes(key)}
          onChange={(v) => onChange({ ...value, [key]: v })}
        />
      ))}
    </div>
  );
};

interface SchemaFieldProps {
  name: string;
  schema: Record<string, unknown>;
  value: unknown;
  required: boolean;
  onChange: (value: unknown) => void;
}

const SchemaField: FC<SchemaFieldProps> = ({ name, schema, value, required, onChange }) => {
  const type = schema.type as string | undefined;
  const description = schema.description as string | undefined;
  const enumValues = schema.enum as string[] | undefined;
  const label = (schema.title as string | undefined) ?? name;

  const handleNestedChange = useCallback(
    (nested: Record<string, unknown>) => {
      onChange(nested);
    },
    [onChange],
  );

  // string with enum -> Select
  if (type === "string" && enumValues && enumValues.length > 0) {
    return (
      <div className="flex flex-col gap-1.5">
        <Label className="font-mono text-xs text-muted-foreground">
          {label}
          {required && <span className="text-destructive"> *</span>}
        </Label>
        <Select value={(value as string) ?? ""} onValueChange={(v) => onChange(v)}>
          <SelectTrigger className="h-8 font-mono text-xs">
            <SelectValue placeholder={`Select ${label}`} />
          </SelectTrigger>
          <SelectContent>
            {enumValues.map((ev) => (
              <SelectItem key={ev} value={ev} className="font-mono text-xs">
                {ev}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
      </div>
    );
  }

  // boolean -> Switch
  if (type === "boolean") {
    return (
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center gap-2">
          <Switch
            checked={!!value}
            onCheckedChange={(checked) => onChange(checked)}
            id={`field-${name}`}
          />
          <Label htmlFor={`field-${name}`} className="font-mono text-xs text-muted-foreground">
            {label}
            {required && <span className="text-destructive"> *</span>}
          </Label>
        </div>
        {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
      </div>
    );
  }

  // number / integer -> Input number
  if (type === "number" || type === "integer") {
    return (
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`field-${name}`} className="font-mono text-xs text-muted-foreground">
          {label}
          {required && <span className="text-destructive"> *</span>}
        </Label>
        <Input
          id={`field-${name}`}
          type="number"
          className="h-8 font-mono text-xs"
          value={value != null ? String(value) : ""}
          onChange={(e) => {
            const v = e.target.value;
            if (v === "") {
              onChange(undefined);
            } else {
              onChange(type === "integer" ? parseInt(v, 10) : parseFloat(v));
            }
          }}
          step={type === "integer" ? 1 : "any"}
        />
        {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
      </div>
    );
  }

  // array -> Textarea for JSON
  if (type === "array") {
    const strValue = value != null ? JSON.stringify(value, null, 2) : "";
    return (
      <div className="flex flex-col gap-1.5">
        <Label htmlFor={`field-${name}`} className="font-mono text-xs text-muted-foreground">
          {label}
          {required && <span className="text-destructive"> *</span>}
        </Label>
        <Textarea
          id={`field-${name}`}
          className="min-h-[80px] font-mono text-xs"
          placeholder="Enter JSON array, e.g. [1, 2, 3]"
          value={strValue}
          onChange={(e) => {
            try {
              const parsed = JSON.parse(e.target.value);
              if (Array.isArray(parsed)) {
                onChange(parsed);
              }
            } catch {
              // Let user keep typing; only update on valid JSON
            }
          }}
        />
        {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
      </div>
    );
  }

  // nested object -> recurse
  if (type === "object" && schema.properties) {
    return (
      <div className="flex flex-col gap-1.5">
        <Label className="font-mono text-xs font-medium text-foreground">
          {label}
          {required && <span className="text-destructive"> *</span>}
        </Label>
        {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
        <div className="rounded-md border border-border/50 p-3">
          <JsonSchemaForm
            schema={schema}
            value={(value as Record<string, unknown>) ?? {}}
            onChange={handleNestedChange}
          />
        </div>
      </div>
    );
  }

  // default: string -> Input text
  return (
    <div className="flex flex-col gap-1.5">
      <Label htmlFor={`field-${name}`} className="font-mono text-xs text-muted-foreground">
        {label}
        {required && <span className="text-destructive"> *</span>}
      </Label>
      <Input
        id={`field-${name}`}
        type="text"
        className="h-8 font-mono text-xs"
        value={(value as string) ?? ""}
        onChange={(e) => onChange(e.target.value)}
      />
      {description && <p className="font-mono text-xs text-muted-foreground/70">{description}</p>}
    </div>
  );
};
