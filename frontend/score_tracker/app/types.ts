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
export function extend_genmap<T>(
  map: GeneratedMap<T>,
  k: string,
  v: T
): GeneratedMap<T> {
  const entries = Object.entries(map);
  let entry_idx = entries.findIndex(([key, _]) => {
    k === key;
  });
  if (entry_idx !== -1) {
    entries[entry_idx] = [k, v];
  } else {
    entries.push([k, v]);
  }

  return Object.fromEntries(entries);
}
export type MemberMap = GeneratedMap<MemberState>;
export type OutcomeMap = GeneratedMap<WagerOutcome>;
export type ChoicesMap = GeneratedMap<number[]>;
