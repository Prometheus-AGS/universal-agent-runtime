/*
 * Copyright 2024 Google LLC
 *
 * Modified by Prometheus AGS for the UAR Zod 4 candidate.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *      https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import type {z} from 'zod';

/**
 * Formats a single Zod issue into a human-readable diagnostic message.
 *
 * Direct attribute extraction is used so that issue details (such as unrecognized
 * property keys or invalid enum options) are preserved even when running in
 * optimized/minified production builds where Zod's internal error map messages
 * may be obfuscated or stripped.
 *
 * @param err Zod validation issue to format.
 * @return Human-readable formatted error message.
 */
export function formatZodIssue(err: z.ZodIssue): string {
  const path = err.path.join('.') || 'root';

  switch (err.code) {
    case 'invalid_union': {
      const unionIssues = err.errors.flat();
      return unionIssues.length > 0
        ? unionIssues.map(formatZodIssue).join('; ')
        : `${path}: Invalid union`;
    }

    case 'unrecognized_keys': {
      const keys = err.keys.map(key => `'${key}'`).join(', ');
      return `${path}: Unrecognized key(s) in object: ${keys}`;
    }

    case 'invalid_type': {
      if (Object.hasOwn(err, 'input')) {
        const received = Array.isArray(err.input) ? 'array' : err.input === null ? 'null' : typeof err.input;
        return `${path}: Expected ${err.expected}, received ${received}`;
      }
      const detail = /^Invalid input: expected (.+)$/.exec(err.message)?.[1];
      return detail ? `${path}: Expected ${detail}` : `${path}: ${err.message}`;
    }

    case 'invalid_value': {
      if (err.values.length < 2) return `${path}: ${err.message}`;
      const options = err.values.map(String).join(' | ');
      return Object.hasOwn(err, 'input')
        ? `${path}: Invalid enum value. Expected ${options}, received '${String(err.input)}'`
        : `${path}: Invalid enum value. Expected ${options}`;
    }

    default:
      return err.message ? `${path}: ${err.message}` : `${path}: Validation error (${err.code})`;
  }
}
