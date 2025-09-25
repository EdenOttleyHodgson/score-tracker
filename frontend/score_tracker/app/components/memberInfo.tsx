import type { MemberState } from "~/backend/types";

export default function MemberInfo({
  memberState,
}: {
  memberState: MemberState;
}) {
  return (
    <div>
      <p>Name: {memberState.name}</p>
      <p>Score: {memberState.score}</p>
    </div>
  );
}
