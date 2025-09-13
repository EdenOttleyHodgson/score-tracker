import type { MemberState } from "~/backend/types";

export default function Member(member_state: MemberState) {
  return (
    <div>
      <p>Name: {member_state.name}</p>
      <p>Score: {member_state.score}</p>
    </div>
  );
}
