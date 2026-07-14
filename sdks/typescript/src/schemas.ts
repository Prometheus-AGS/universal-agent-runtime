import { z } from "zod";

const jsonSchema: z.ZodType<unknown> = z.unknown();
export const chatCompletionSchema = z.object({
  id: z.string().optional(),
  choices: z.array(z.object({
    index: z.number().optional(),
    message: z.object({ role: z.string().optional(), content: z.string().nullable().optional() }).passthrough(),
    finish_reason: z.string().nullable().optional(),
  }).passthrough()),
}).passthrough();
export const runResponseSchema = z.object({
  run_id: z.string(),
  stream_url: z.string(),
  resumed_from_run_id: z.string().optional(),
}).passthrough();
export const toolResultSchema = z.object({
  result: jsonSchema.optional(), error: z.string().nullable().optional(), duration_ms: z.number(), success: z.boolean(),
}).passthrough();
export const embeddingResponseSchema = z.object({
  data: z.array(z.object({ embedding: z.array(z.number()), index: z.number().optional() }).passthrough()),
  model: z.string().optional(),
}).passthrough();
export const knowledgeBaseSchema = z.object({
  id: z.string(), name: z.string(), description: z.string().nullable().optional(),
  config: z.record(z.string(), z.unknown()).optional(), created_at: z.string(), updated_at: z.string(),
}).passthrough();
export const documentSchema = z.object({ id: z.string() }).passthrough();
export const searchResponseSchema = z.object({ results: z.array(z.object({
  content: z.string(), score: z.number(), metadata: z.unknown().optional(), document_id: z.string().nullable().optional(),
}).passthrough()) }).passthrough();
export const ingestResponseSchema = z.object({ success: z.boolean(), chunk_count: z.number() }).passthrough();
export const checkpointListSchema = z.object({ run_id: z.string(), checkpoints: z.array(z.unknown()) }).passthrough();
export const cancelResponseSchema = z.object({ cancelled: z.boolean() });
