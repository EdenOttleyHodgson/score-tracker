import type {
  MemberState,
  Pot,
  Wager,
  WagerOutcome,
  RoomCode,
  ID,
} from "./types.ts";

type RustVariantType = "Unit" | "Struct" | "Tuple";
interface Message<T extends ServerMessage | ClientMessage> {
  kind: string;
  variant_type: RustVariantType;
  data: T;
}
export type ServerMessage =
  | SynchronizeRoom
  | UserJoined
  | RoomDeleted
  | UserRemoved
  | PotCreated
  | PotJoined
  | PotResolved
  | WagerCreated
  | WagerJoined
  | WagerResolved
  | ScoreChanged
  | AdminGranted
  | ErrorMsg;
export type ClientMessage =
  | CreateRoom
  | JoinRoom
  | LeaveRoom
  | RemoveFromRoom
  | DeleteRoom
  | RequestAdmin
  | BlessScore
  | RemoveScore
  | GiveScore
  | TransferScore
  | CreatePot
  | JoinPot
  | ResolvePot
  | CreateWager
  | JoinWager
  | ResolveWager;

interface SynchronizeRoom {
  variant_type: "Struct";
  members: MemberState[];
  pots: Pot[];
  wager: Wager[];
}
interface UserJoined {
  variant_type: "Struct";
  name: string;
  id: ID;
}

interface RoomDeleted {
  variant_type: "Unit";
}
interface UserRemoved {
  variant_type: "Tuple";
  id: ID;
}
interface PotCreated {
  variant_type: "Tuple";
  pot: Pot;
}
interface PotJoined {
  variant_type: "Struct";
  pot_id: ID;
  user_id: ID;
}
interface PotResolved {
  variant_type: "Tuple";
  id: ID;
}
interface WagerCreated {
  variant_type: "Tuple";
  wager: Wager;
}
interface WagerJoined {
  variant_type: "Struct";
  wager_id: ID;
  user_id: ID;
  outcome_id: ID;
  amount: number;
}
interface WagerResolved {
  variant_type: "Tuple";
  id: ID;
}
interface ScoreChanged {
  variant_type: "Struct";
  user_id: ID;
  new_amount: number;
}
interface AdminGranted {
  variant_type: "Unit";
}
interface ErrorMsg {
  variant_type: "Struct";
  description: string;
  display_to_user: boolean;
}

interface CreateRoom {
  variant_type: "Struct";
  code: RoomCode;
  admin_pass: string;
}
interface JoinRoom {
  variant_type: "Tuple";
  code: RoomCode;
  name: string;
}
interface LeaveRoom {
  variant_type: "Tuple";
  code: RoomCode;
}
interface RemoveFromRoom {
  variant_type: "Tuple";
  code: RoomCode;
  id: ID;
}
interface DeleteRoom {
  variant_type: "Tuple";
  code: RoomCode;
}
interface RequestAdmin {
  variant_type: "Struct";
  room: RoomCode;
  password: String;
}
interface BlessScore {
  variant_type: "Struct";
  to: ID;
  amount: number;
}
interface RemoveScore {
  variant_type: "Struct";
  from: ID;
  amount: number;
}
interface GiveScore {
  variant_type: "Struct";
  to: ID;
  amount: number;
}
interface TransferScore {
  variant_type: "Struct";
  from: ID;
  to: ID;
  amount: number;
}
interface CreatePot {
  variant_type: "Struct";
  room_code: RoomCode;
  score_requirement: number;
  description: String;
}

interface JoinPot {
  variant_type: "Struct";
  room_code: RoomCode;
  pot_id: ID;
}

interface ResolvePot {
  variant_type: "Struct";
  room_id: RoomCode;
  pot_id: ID;
  winner: ID;
}
interface CreateWager {
  variant_type: "Struct";
  room_id: RoomCode;
  name: String;
  outcomes: [WagerOutcome];
}
interface JoinWager {
  kind: "JoinWager";
  variant_type: "Struct";
  room_id: RoomCode;
  wager_id: ID;
  outcome_id: ID;
  amount: number;
}
interface ResolveWager {
  kind: "ResolveWager";
  variant_type: "Struct";
  room_id: RoomCode;
  wager_id: ID;
  outcome_id: ID;
}

export function parse_message(kind: string, values: any): ServerMessage {
  let msg = parse_message_unchecked(kind, values);
}

function parse_message_unchecked(kind: string, values: any): ServerMessage {
  // switch (kind) {
  //   case "SynchronizeRoom":
  //     return {
  //       kind,
  //       variant_type: "Struct",
  //       members: values.members,
  //       pots: values.pots,
  //       wager: values.wagers,
  //     };
  //   case "UserJoined":
  //     return {
  //       kind,
  //       variant_type: "Struct",
  //       name: values.name,
  //       id: values.id
  //     }
  //   case "RoomDeleted":
  //     return {
  //       kind,
  //       variant_type: "Unit"
  //     }
  //   case "UserRemoved":
  //     return {
  //       kind,
  //       variant_type: "Tuple",
  //       id: values.id,
  //     }
  //   case "PotCreated":
  //     return {
  //       kind,
  //       variant_type: "Tuple",
  //       pot: values.pot,
  //     }
  //   case "PotJoined":
  //     return {
  //       kind,
  //       variant_type: "Struct",
  //       pot_id: values.pot_id,
  //       user_id: values.user_id
  //     }
  //   case "PotResolved":
  //     return {
  //       kind,
  //       variant_type: "Tuple"
  //         id: values.id
  //     }
  //   case "WagerCreated":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   case "WagerJoined":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   case "WagerResolved":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   case "ScoreChanged":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   case "AdminGranted":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   case "ErrorMsg":
  //     return {
  //       kind,
  //       variant_type: ""
  //     }
  //   default:
  //     console.error(
  //       `Could not parse message from server: Invalid kind: ${kind}`
  //     );
  // }
}

export function to_sendable_msg(msg: Message<ClientMessage>): any {
  let keys = Object.keys(msg).filter((x) => x != "kind" && x != "variant_type");
  switch (msg.variant_type) {
    case "Struct":
      return { [msg.kind]: keys };
    case "Unit":
      return msg.kind;
    case "Tuple":
      if (keys.length == 1) {
        return { [msg.kind]: keys[0] };
      } else {
        return { [msg.kind]: keys };
      }
  }
}
