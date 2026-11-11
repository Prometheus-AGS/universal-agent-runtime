import {z} from 'zod';

// A2UI schemas are also parsed directly by renderers. Keep those parsers
// usable under the runtime's Content Security Policy without dynamic code.
z.config({jitless: true});

export {z};

export function toWireJsonSchema(schema: z.ZodType): Record<string, unknown> {
  return z.toJSONSchema(schema, {
    io: 'input',
    target: 'draft-2020-12',
    reused: 'inline',
    unrepresentable: 'any',
    override: ({zodSchema, jsonSchema}) => {
      // The previous converter closed stripping objects in exported
      // catalogs. Keep that wire behavior while emitting input types.
      const definition = zodSchema._zod.def;
      if (definition.type === 'object' && !definition.catchall) {
        jsonSchema.additionalProperties = false;
      }
      if (definition.type === 'object' && definition.catchall?._zod.def.type === 'unknown') {
        jsonSchema.additionalProperties = true;
      }
      if (definition.type === 'number' && jsonSchema.type === 'integer') {
        const checks = definition.checks ?? [];
        // Zod 4 uses safe integers internally. The catalog wire format
        // carries only the bounds its author explicitly supplied.
        if (
          jsonSchema.minimum === Number.MIN_SAFE_INTEGER &&
          !checks.some(check => check._zod.def.check === 'greater_than')
        ) {
          delete jsonSchema.minimum;
        }
        if (
          jsonSchema.maximum === Number.MAX_SAFE_INTEGER &&
          !checks.some(check => check._zod.def.check === 'less_than')
        ) {
          delete jsonSchema.maximum;
        }
      }
      if (definition.type === 'prefault' && '_prefault' in jsonSchema) {
        jsonSchema.default = jsonSchema._prefault;
        delete jsonSchema._prefault;
      }
    },
  });
}

/** The schema carrying a wrapped field's input or output structure. */
export function wrappedSchema(
  schema: z.ZodType,
  followLazy = true,
): z.ZodType | undefined {
  if (
    schema instanceof z.ZodOptional ||
    schema instanceof z.ZodExactOptional ||
    schema instanceof z.ZodNullable ||
    schema instanceof z.ZodDefault ||
    schema instanceof z.ZodPrefault ||
    schema instanceof z.ZodCatch ||
    schema instanceof z.ZodReadonly
  ) {
    return schema.unwrap() as z.ZodType;
  }
  if (schema instanceof z.ZodPipe) {
    // A preprocess/guard pipe starts with a transform or an unconstrained
    // input. Its output schema is the first useful shape in that case.
    if (
      schema.in instanceof z.ZodTransform ||
      schema.in instanceof z.ZodAny ||
      schema.in instanceof z.ZodUnknown
    ) {
      return schema.out instanceof z.ZodType ? schema.out : undefined;
    }
    return schema.in instanceof z.ZodType ? schema.in : undefined;
  }
  if (followLazy && schema instanceof z.ZodLazy) {
    return schema.unwrap() as z.ZodType;
  }
  return undefined;
}

export function unwrapSchema(schema: z.ZodType): z.ZodType {
  const seen = new Set<z.ZodType>();
  let current = schema;
  while (!seen.has(current)) {
    seen.add(current);
    const next = wrappedSchema(current);
    if (!next) break;
    current = next;
  }
  return current;
}
