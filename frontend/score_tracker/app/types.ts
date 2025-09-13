import type { MemberState, WagerOutcome } from "./backend/types";

export type GeneratedMap<T> = { [k: string]: T };
export function get_from_genmap<T>(
  map: GeneratedMap<T>,
  k: string
): T | undefined {
  let found = Object.entries(map).find((x) => x[0] === k);
  if (found) {
    return found[1];
  } else {
    return undefined;
  }
}
export type MemberMap = GeneratedMap<MemberState>;
export type OutcomeMap = GeneratedMap<WagerOutcome>;
export type ChoicesMap = GeneratedMap<number[]>;
