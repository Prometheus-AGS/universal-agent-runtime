import type { TeamMemberSpec } from "@/platform/entities";

export type ParsedInput = { ok: true; value: unknown } | { ok: false; message: string };

export function parseInput(source: string): ParsedInput {
  try {
    return { ok: true, value: JSON.parse(source) as unknown };
  } catch (failure) {
    return { ok: false, message: failure instanceof Error ? failure.message : "The input is not valid JSON." };
  }
}

/** A whole number inside the role's declared minimum and maximum, or the reason it is not. */
export function slotProblem(text: string, member: Pick<TeamMemberSpec, "min" | "max">): string | null {
  if (!/^\d+$/.test(text.trim())) return "Enter a whole number.";
  const count = Number(text);
  if (count < member.min || count > member.max) {
    return member.min === member.max ? `This role must have exactly ${member.min}.` : `Choose a number between ${member.min} and ${member.max}.`;
  }
  return null;
}
