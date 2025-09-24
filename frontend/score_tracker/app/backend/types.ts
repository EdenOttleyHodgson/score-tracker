export type ClientMessage =
  | {
      admin_pass: string;
      code: RoomCode;
      kind: "CreateRoom";
      [k: string]: unknown;
    }
  | {
      code: RoomCode;
      kind: "JoinRoom";
      name: string;
      [k: string]: unknown;
    }
  | {
      kind: "LeaveRoom";
      room_code: RoomCode;
      [k: string]: unknown;
    }
  | {
      code: RoomCode;
      id: number;
      kind: "RemoveFromRoom";
      [k: string]: unknown;
    }
  | {
      kind: "DeleteRoom";
      room_code: RoomCode;
      [k: string]: unknown;
    }
  | {
      kind: "RequestAdmin";
      password: string;
      room: RoomCode;
      [k: string]: unknown;
    }
  | {
      amount: number;
      kind: "BlessScore";
      to: number;
      [k: string]: unknown;
    }
  | {
      amount: number;
      from: number;
      kind: "RemoveScore";
      [k: string]: unknown;
    }
  | {
      amount: number;
      kind: "GiveScore";
      to: number;
      [k: string]: unknown;
    }
  | {
      amount: number;
      from: number;
      kind: "TransferScore";
      to: number;
      [k: string]: unknown;
    }
  | {
      description: string;
      kind: "CreatePot";
      room_code: RoomCode;
      score_requirement: number;
      [k: string]: unknown;
    }
  | {
      kind: "JoinPot";
      pot_id: number;
      room_code: RoomCode;
      [k: string]: unknown;
    }
  | {
      kind: "ResolvePot";
      pot_id: number;
      room_id: RoomCode;
      winner: number;
      [k: string]: unknown;
    }
  | {
      kind: "CreateWager";
      name: string;
      outcomes: WagerOutcome[];
      room_id: RoomCode;
      [k: string]: unknown;
    }
  | {
      amount: number;
      kind: "JoinWager";
      outcome_id: number;
      room_id: RoomCode;
      wager_id: number;
      [k: string]: unknown;
    }
  | {
      kind: "ResolveWager";
      outcome_id: number;
      room_id: RoomCode;
      wager_id: number;
      [k: string]: unknown;
    };

export type RoomCode = string;

export interface WagerOutcome {
  description: string;
  id: number;
  name: string;
  odds: number;
  [k: string]: unknown;
}

export interface MemberState {
  current_pots: number[];
  current_wagers: number[];
  id: number;
  name: string;
  score: number;
  [k: string]: unknown;
}

export interface Pot {
  description: string;
  participants: number[];
  pot_id: number;
  score_requirement: number;
  total_score: number;
  [k: string]: unknown;
}

export type ServerMessage =
  | {
      kind: "SynchronizeRoom";
      members: MemberState[];
      pots: Pot[];
      requester_id: number;
      wager: Wager[];
      [k: string]: unknown;
    }
  | {
      code: RoomCode;
      kind: "RoomCreated";
      [k: string]: unknown;
    }
  | {
      id: number;
      kind: "UserJoined";
      name: string;
      [k: string]: unknown;
    }
  | {
      kind: "RoomDeleted";
      [k: string]: unknown;
    }
  | {
      id: number;
      kind: "UserRemoved";
      [k: string]: unknown;
    }
  | {
      kind: "PotCreated";
      pot: Pot;
      [k: string]: unknown;
    }
  | {
      kind: "PotJoined";
      pot_id: number;
      user_id: number;
      [k: string]: unknown;
    }
  | {
      id: number;
      kind: "PotResolved";
      [k: string]: unknown;
    }
  | {
      kind: "WagerCreated";
      wager: Wager;
      [k: string]: unknown;
    }
  | {
      amount: number;
      kind: "WagerJoined";
      outcome_id: number;
      user_id: number;
      wager_id: number;
      [k: string]: unknown;
    }
  | {
      id: number;
      kind: "WagerResolved";
      [k: string]: unknown;
    }
  | {
      kind: "ScoreChanged";
      new_amount: number;
      user_id: number;
      [k: string]: unknown;
    }
  | {
      kind: "AdminGranted";
      [k: string]: unknown;
    }
  | {
      description: string;
      display_to_user: boolean;
      kind: "Error";
      [k: string]: unknown;
    };

export interface Wager {
  id: number;
  name: string;
  outcomes: {
    [k: string]: WagerOutcome;
  };
  participant_bets: {
    /**
     * This interface was referenced by `undefined`'s JSON-Schema definition
     * via the `patternProperty` "^\d+$".
     */
    [k: string]: number;
  };
  participant_choices: {
    /**
     * This interface was referenced by `undefined`'s JSON-Schema definition
     * via the `patternProperty` "^\d+$".
     */
    [k: string]: number[];
  };
  [k: string]: unknown;
}
/**
 * This interface was referenced by `undefined`'s JSON-Schema definition
 * via the `patternProperty` "^\d+$".
 */