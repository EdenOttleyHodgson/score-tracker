import type { MemberState } from "./backend/types";
import type { GeneratedMap } from "./types";

export function unreachable(x: never): never {
  throw new Error("Unreachable code called");
}

export function initMemberState(id: number, name: string): MemberState {
  return {
    id,
    name,
    current_pots: [],
    current_wagers: [],
    score: 0,
  };
}

export function makeMemberMap(array: MemberState[]): GeneratedMap<MemberState> {
  return Object.fromEntries(array.map((member) => [member.id, member]));
}
