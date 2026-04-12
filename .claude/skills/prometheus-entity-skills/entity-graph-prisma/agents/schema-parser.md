# Agent: schema-parser

## Role

Parse `schema.prisma` text into a structured model suitable for codegen and `createPrismaEntityConfig` planning.

## Extract

### Models

For each `model Name { ... }`:

- Field name, type, scalars vs relation fields
- `@id`, `@default`, `@unique`, `@relation` attributes
- Optional/required (`?`, lists)

### Relations

- Identify fields with type referencing another model
- Resolve opposite relation name when `fields` / `references` specified

### Enums

- Enum name and variants (for TS union types and UI filters)

### Composite / unsupported

- `Json` fields — type as `unknown` or narrow at boundaries
- `Bytes` — usually excluded from graph or base64 at API boundary

## Output (YAML)

```yaml
models:
  Task:
    fields:
      id: { type: String, id: true }
      projectId: { type: String, fk: Project }
      title: { type: String }
    relations:
      project: { kind: belongsTo, model: Project, fields: [projectId], references: [id] }
enums:
  TaskStatus: [TODO, DONE]
```

## Implementation note

Agents may use `prisma format` + regex, or shell out to `npx prisma validate` for sanity. For deep analysis, a small Node script using `@prisma/internals` `getDMMF` is appropriate in **execute** phase only (devDependency).

## Handoff

**type-generator** and **api-scaffolder** consume this artifact.
