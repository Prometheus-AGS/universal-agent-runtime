// Shared answer parsing for manual BDD oracles. Formatting is tolerated; values are not.
import assert from 'node:assert/strict';

// Reasoning is never answer text: drop everything through the last closing tag
// (an opening tag may be absent), then any closed blocks that remain.
export function stripReasoning(raw) {
  const text = String(raw ?? '');
  const closing = text.lastIndexOf('</think>');
  const tail = closing >= 0 ? text.slice(closing + '</think>'.length) : text;
  return tail.replace(/<think>[\s\S]*?<\/think>/g, '').replace(/<think>[\s\S]*$/, '').trim();
}

// One JSON answer: the whole reply, or exactly one fenced block anywhere in it.
export function parseAnswerJson(raw) {
  const final = stripReasoning(raw);
  try { return JSON.parse(final); } catch { /* fall through to a fenced block */ }
  const blocks = [...final.matchAll(/```(?:json)?\s*\n?([\s\S]*?)```/g)];
  assert.equal(blocks.length, 1, `Require one unambiguous JSON answer: ${final.slice(0, 300)}`);
  return JSON.parse(blocks[0][1]);
}

// A citation must start with the retrieved marker, e.g. "[1]" or "[1] (file) — note".
export const citesMarker = (answerCitation, marker) => String(answerCitation ?? '').trim().startsWith(`[${marker}]`);
