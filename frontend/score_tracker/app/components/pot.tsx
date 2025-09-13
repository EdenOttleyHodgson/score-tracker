import type { MemberState, Pot } from "~/backend/types";

type PotProps = {
  pot: Pot;
  member_map: Map<number, MemberState>;
};

export default function Pot({ pot, member_map }: PotProps) {
  const participant_items = pot.participants.map((p_id) => {
    const member = member_map.get(p_id);
    if (member) {
      return <li key={p_id}> {member.name} </li>;
    } else {
      console.error("Nonexistent member in pot!");
      return;
    }
  });

  return (
    <div>
      <p>{pot.description}</p>
      <p>Total Score: {pot.total_score}</p>
      <p>Join Requirement: {pot.score_requirement}</p>
      <p>Participants: </p>
      <ul>{participant_items}</ul>
    </div>
  );
}
