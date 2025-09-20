import { parseMessage, useBackendSocket } from "~/backend";
import type { Route } from "./+types/room";
import { useEffect, useState } from "react";
import type {
  ClientMessage,
  MemberState,
  Pot,
  ServerMessage,
  Wager,
} from "~/backend/types";
import { initMemberState, unreachable } from "~/utils";
import { useNavigate, useOutletContext, useSearchParams } from "react-router";
import { extend_genmap, get_from_genmap } from "~/types";
import WagerComponent from "~/components/wager";
import PotComponent from "~/components/pot";
import Member from "~/components/member";
import type { LayoutContext } from "./layout";
export async function clientLoader({ params }: Route.LoaderArgs) {
  return {
    code: params.roomCode,
  };
}
type ReactSetter<T> = React.Dispatch<React.SetStateAction<T | undefined>>;

export default function Room({ loaderData }: Route.ComponentProps) {
  const [isAdmin, setIsAdmin] = useState(false);
  const [members, setMembers] = useState<MemberState[] | undefined>(undefined);
  const [wagers, setWagers] = useState<Wager[] | undefined>(undefined);
  const [pots, setPots] = useState<Pot[] | undefined>(undefined);
  const { adminPass, displayName } = useOutletContext<LayoutContext>();
  const [socket, sendMessage] = useBackendSocket(handle_message, () => {
    navigator("./noServerConnection");
  });

  const navigator = useNavigate();
  const [searchParams, _] = useSearchParams();

  const ready = members && wagers && pots;
  function handle_message(msg: ServerMessage) {
    console.log("Handling message");
    switch (msg.kind) {
      case "SynchronizeRoom":
        setMembers(msg.members);
        setWagers(msg.wager);
        setPots(msg.pots);

        break;
      case "UserJoined":
        setMembers((members) =>
          members
            ? [...members, initMemberState(msg.id, msg.name)]
            : [initMemberState(msg.id, msg.name)]
        );
        break;
      case "UserRemoved":
        setMembers((members) =>
          members ? members.filter((member) => member.id != msg.id) : []
        );
        break;
      case "RoomDeleted":
        alert("Room has been deleted by an admin.");
        useNavigate()("/");
        break;
      case "ScoreChanged":
        if (members) {
          setMembers(
            members.map((member) => {
              if (member.id === msg.user_id) {
                return { ...member, score: msg.new_amount };
              } else {
                return member;
              }
            })
          );
        } else {
          console.error("Members list undefined, bad score changed message");
        }

        break;
      case "AdminGranted":
        setIsAdmin(true);
        break;
      case "WagerCreated":
        setWagers((wagers) => (wagers ? [...wagers, msg.wager] : [msg.wager]));
        break;
      case "WagerJoined":
        if (wagers) {
          setWagers(wagers.map((wager) => update_wager(wager, msg)));
        } else {
          console.error("Wagers are undefined but a wager has been joined!");
        }
        break;
      case "WagerResolved":
        if (wagers) {
          setWagers(wagers.filter((wager) => wager.id !== msg.id));
        } else {
          console.error("Wagers are undefined but a wager has been resolved");
        }
        break;
      case "PotCreated":
        setPots((pots) => (pots ? [...pots, msg.pot] : [msg.pot]));
        break;
      case "PotJoined":
        if (pots) {
          setPots(
            pots.map((pot) => {
              if (pot.pot_id === msg.pot_id) {
                return {
                  ...pot,
                  total_score: (pot.total_score += pot.score_requirement),
                  participants: [...pot.participants, msg.user_id],
                };
              } else {
                return pot;
              }
            })
          );
        } else {
          console.error("Pot joined despite pots being undefined");
        }
        break;
      case "PotResolved":
        if (pots) {
          setPots(pots.filter((pot) => pot.pot_id !== msg.id));
        } else {
          console.error("Pot resolved despite pots being undefined");
        }

        break;
      case "Error":
        if (msg.display_to_user) {
          alert(msg.description);
        } else {
          console.error(msg.description);
        }
        break;
      default:
        return unreachable(msg);
    }
  }
  useEffect(() => {
    if (searchParams.get("create")) {
      sendMessage({
        kind: "CreateRoom",
        admin_pass: adminPass.value,
        code: loaderData.code,
      });
    }
    sendMessage({
      kind: "JoinRoom",
      code: loaderData.code,
      name: displayName.value,
    });
    return () => {
      sendMessage({ kind: "LeaveRoom", room_code: loaderData.code });
    };
  }, []);

  if (ready) {
    const memberMap = new Map(members.map((member) => [member.id, member]));
    const memberComponents = members.map((member) => (
      <Member member_state={member} />
    ));
    const wagerComponents = wagers.map((wager) => (
      <WagerComponent wager={wager} member_map={memberMap} />
    ));
    const potComponents = pots.map((pot) => (
      <PotComponent pot={pot} member_map={memberMap} />
    ));

    return (
      <div>
        {wagerComponents}
        <br />
        {memberComponents}
        <br />
        {potComponents}
        <br />
        <button
          onClick={() =>
            sendMessage({
              kind: "CreateWager",
              room_id: loaderData.code,
              name: "plarka",
              outcomes: [{ description: "tingas", name: "My", odds: 50 }],
            })
          }
        ></button>
      </div>
    );
  } else {
    return <p>waiting</p>;
  }
}

function update_wager(
  wager: Wager,
  msg: { wager_id: number; outcome_id: number; user_id: number; amount: number }
): Wager {
  if (wager.id === msg.wager_id) {
    const relevantChoices = get_from_genmap(
      wager.participant_choices,
      msg.outcome_id.toString()
    );
    if (relevantChoices) {
      const participant_choices = extend_genmap(
        wager.participant_choices,
        msg.outcome_id.toString(),
        [...relevantChoices, msg.user_id]
      );
      const participant_bets = extend_genmap(
        wager.participant_bets,
        msg.user_id.toString(),
        msg.amount
      );
      return { ...wager, participant_choices, participant_bets };
    } else {
      console.error("Choice not present in wager");
      return wager;
    }
  } else {
    return wager;
  }
}
