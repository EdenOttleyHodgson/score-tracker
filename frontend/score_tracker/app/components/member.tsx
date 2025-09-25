import type { MemberState } from "~/backend/types";
import MemberInfo from "./memberInfo";

type MemberCallback = (id: number) => void;

export interface MemberProps {
  memberState: MemberState;
  isAdmin: boolean;
  onGive: MemberCallback;
  onKick: MemberCallback;
  onBless: MemberCallback;
  onRemoveScore: MemberCallback;
}

export default function Member({
  memberState,
  isAdmin,
  onGive,
  onKick,
  onBless,
  onRemoveScore,
}: MemberProps) {
  return (
    <div>
      <MemberInfo memberState={memberState} />
      <button onClick={() => onGive(memberState.id)}>Give Score</button>
      {isAdmin && (
        <div className={"flex gap-5"}>
          <button onClick={() => onKick(memberState.id)}>Kick</button>
          <button onClick={() => onBless(memberState.id)}>Bless Score</button>
          <button onClick={() => onRemoveScore(memberState.id)}>
            Remove Score
          </button>
        </div>
      )}
    </div>
  );
}
