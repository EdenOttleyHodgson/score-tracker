import type { MemberState, Wager, WagerOutcome } from "~/backend/types";
import { get_from_genmap, type GeneratedMap, type MemberMap } from "~/types";

interface WagerProps {
  wager: Wager;
  member_map: Map<number, MemberState>;
  onOutcomeClicked: (id: number) => void;
}

export default function Wager({
  wager,
  member_map,
  onOutcomeClicked,
}: WagerProps) {
  let wager_members = Object.entries(wager.participant_choices).flatMap(
    ([choice_id, user_set]) =>
      user_set.map((user) =>
        make_wager_member(
          user,
          choice_id,
          member_map,
          wager.outcomes,
          wager.participant_bets
        )
      )
  );
  let wager_member_items = wager_members.map((x) => (
    <li key={x.id}>
      <WagerMember {...x} />
    </li>
  ));
  let wager_outcome_items = Object.entries(wager.outcomes).map(
    ([id, outcome]) => (
      <li key={id}>
        <WagerOutcome outcome={outcome} onOutcomeClicked={onOutcomeClicked} />
      </li>
    )
  );
  return (
    <div>
      <h2>Name: {wager.name}</h2>
      <ul>{wager_outcome_items}</ul>
      <ul>{wager_member_items}</ul>
    </div>
  );
}
interface WagerMemberProps {
  id: number;
  username: string;
  choice_name: string;
  amount_wagered: number;
}
function WagerMember({
  username,
  choice_name,
  amount_wagered,
}: WagerMemberProps) {
  return (
    <p>
      {username} -- {choice_name} -- {amount_wagered}
    </p>
  );
}

function make_wager_member(
  p_id: number,
  choice_id: string,
  member_map: Map<number, MemberState>,
  outcomes: GeneratedMap<WagerOutcome>,
  amounts: GeneratedMap<number>
): WagerMemberProps {
  console.log(
    "making wager member",
    p_id,
    choice_id,
    member_map,
    outcomes,
    amounts
  );
  const choice_name = get_from_genmap(outcomes, choice_id)?.name;
  if (!choice_name)
    throw new Error(`Choice with id ${choice_id} not found in outcome map`);
  const p_id_str = p_id.toString();
  const username = member_map.get(p_id)?.name;
  if (!username)
    throw new Error(`User with id ${p_id} not found in member map`);
  const amount_wagered = get_from_genmap(amounts, p_id_str);
  if (!amount_wagered)
    throw new Error(`User with id ${p_id} not present in amount map`);
  return { id: p_id, username, choice_name, amount_wagered };
}

function WagerOutcome({
  outcome,
  onOutcomeClicked,
}: {
  outcome: WagerOutcome;
  onOutcomeClicked: (id: number) => void;
}) {
  return (
    <div className={"flex gap-5"}>
      <h3>{outcome.name}</h3>
      <p>{outcome.description}</p>
      <p>Odds:{outcome.odds}</p>
      <button
        onClick={() => {
          onOutcomeClicked(outcome.id);
        }}
      >
        Join
      </button>
    </div>
  );
}
