import { parseMessage, useBackendSocket } from "~/backend";
import type { Route } from "./+types/room";
import React, { useCallback, useEffect, useState } from "react";
import type {
  ClientMessage,
  MemberState,
  Pot,
  ServerMessage,
  Wager,
} from "~/backend/types";
import { initMemberState, promptAmount, unreachable } from "~/utils";
import {
  NavLink,
  useNavigate,
  useOutletContext,
  useSearchParams,
} from "react-router";
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
type ReactSetterDefined<T> = React.Dispatch<React.SetStateAction<T>>;

export default function Room({ loaderData }: Route.ComponentProps) {
  const [isAdmin, setIsAdmin] = useState(false);
  const [members, setMembers] = useState<MemberState[] | undefined>(undefined);
  const [wagers, setWagers] = useState<Wager[] | undefined>(undefined);
  const [pots, setPots] = useState<Pot[] | undefined>(undefined);
  const [adminPass, setAdminPass] = useState("");
  const [selfID, setSelfID] = useState<number | undefined>(undefined);
  const [removed, setRemoved] = useState(false);
  const { adminPass: newAdminPass } = useOutletContext<LayoutContext>();

  const [socket, sendMessage] = useBackendSocket(
    (msg) =>
      handleMessage(
        msg,
        members,
        wagers,
        pots,
        setMembers,
        setWagers,
        setPots,
        setSelfID,
        setIsAdmin,
        setRemoved
      ),
    () => {
      navigate("/noServerConnection");
    }
  );

  const navigate = useNavigate();
  const [searchParams, _] = useSearchParams();
  const displayName = searchParams.get("name") || "User";

  const ready = members && wagers && pots && selfID !== undefined;

  useEffect(() => {
    if (searchParams.get("create")) {
      sendMessage({
        kind: "CreateRoom",
        admin_pass: newAdminPass.value,
        code: loaderData.code,
      });
    }
    sendMessage({
      kind: "JoinRoom",
      code: loaderData.code,
      name: displayName,
    });
    return () => {
      sendMessage({ kind: "LeaveRoom", room_code: loaderData.code });
      setRemoved(false);
    };
  }, []);

  if (ready) {
    const memberMap = new Map(members.map((member) => [member.id, member]));
    const memberComponents = members.map((member) => (
      <Member
        memberState={member}
        isAdmin={isAdmin}
        onBless={(id) => {
          const amount = promptAmount();
          if (amount) {
            sendMessage({ kind: "BlessScore", amount, to: id });
          }
        }}
        onRemoveScore={(id) => {
          const amount = promptAmount();
          if (amount) {
            sendMessage({ kind: "RemoveScore", amount, from: id });
          }
        }}
        onGive={(id) => {
          const amount = promptAmount();
          if (amount) {
            sendMessage({ kind: "GiveScore", amount, to: id });
          }
        }}
        onKick={(id) =>
          sendMessage({ kind: "RemoveFromRoom", code: loaderData.code, id })
        }
      />
    ));

    const wagerComponents = wagers.map((wager) => (
      <WagerComponent
        wager={wager}
        member_map={memberMap}
        onOutcomeClicked={(outcome_id) => {
          const amount = promptAmount();
          if (amount) {
            sendMessage({
              kind: "JoinWager",
              amount,
              wager_id: wager.id,
              outcome_id,
              room_id: loaderData.code,
            });
          }
        }}
      />
    ));
    const potComponents = pots.map((pot) => (
      <PotComponent
        pot={pot}
        member_map={memberMap}
        onJoinClicked={() => {
          sendMessage({
            kind: "JoinPot",
            pot_id: pot.pot_id,
            room_code: loaderData.code,
          });
        }}
      />
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
          onClick={() => {
            sendMessage({
              kind: "CreateWager",
              room_id: loaderData.code,
              name: "plarka",
              outcomes: [
                { id: 0, description: "tingas", name: "My", odds: 50 },
              ],
            });
            sendMessage({
              kind: "CreatePot",
              description: "my slorkatingas",
              room_code: loaderData.code,
              score_requirement: 100,
            });
            sendMessage({ kind: "BlessScore", amount: 1000, to: selfID });
          }}
        >
          my grarkatingas
        </button>
        <input
          onChange={(ev) => {
            setAdminPass(ev.target.value);
          }}
        />
        <button
          onClick={() => {
            sendMessage({
              kind: "RequestAdmin",
              room: loaderData.code,
              password: adminPass,
            });
          }}
        >
          admin request
        </button>
        {isAdmin && (
          <button
            onClick={() =>
              sendMessage({ kind: "DeleteRoom", room_code: loaderData.code })
            }
          >
            Delete Room
          </button>
        )}
        <button onClick={() => sendMessage({ kind: "Debug" })}>Debug</button>
        {removed && (
          <NavLink to="/">
            You have been removed from the room.Click to navigate back to the
            main menu.
          </NavLink>
        )}
      </div>
    );
  } else {
    return (
      <div>
        <p>waiting</p>
      </div>
    );
  }
}

function update_wager(
  wager: Wager,
  msg: { wager_id: number; outcome_id: number; user_id: number; amount: number }
): Wager {
  console.log("updating wager:", wager);
  if (wager.id === msg.wager_id) {
    const outcome = wager.outcomes[msg.outcome_id.toString()];
    if (outcome) {
      const newBets = { ...wager.participant_bets };
      newBets[msg.user_id] = msg.amount;
      const newChoices = { ...wager.participant_choices };
      const relevantChoosers = newChoices[msg.outcome_id.toString()];
      if (relevantChoosers) {
        newChoices[msg.outcome_id.toString()] = [
          ...relevantChoosers,
          msg.user_id,
        ];
      } else {
        newChoices[msg.outcome_id.toString()] = [msg.user_id];
      }
      return {
        ...wager,
        participant_choices: newChoices,
        participant_bets: newBets,
      };
    } else {
      console.error("Outcome doesnt exist in wager");
    }
  }
  return wager;
}

function handleMessage(
  msg: ServerMessage,
  members: MemberState[] | undefined,
  wagers: Wager[] | undefined,
  pots: Pot[] | undefined,
  setMembers: ReactSetter<MemberState[]>,
  setWagers: ReactSetter<Wager[]>,
  setPots: ReactSetter<Pot[]>,
  setSelfID: ReactSetter<number>,
  setIsAdmin: ReactSetterDefined<boolean>,
  setRemoved: ReactSetterDefined<boolean>
) {
  console.log("Handling message");
  //ugly hack, but if you're recieving a non-error message you're still in the room.
  if (msg.kind !== "Error") {
    setRemoved(false);
  }
  switch (msg.kind) {
    case "RoomCreated":
      break;

    case "SynchronizeRoom":
      setMembers(msg.members);
      setWagers(msg.wager);
      setPots(msg.pots);
      setSelfID(msg.requester_id);
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
      setRemoved(true);
      break;
    case "ScoreChanged":
      console.log(members);
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
    case "RecieverLeft":
      console.log("reciever left");
      setRemoved(true);
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
