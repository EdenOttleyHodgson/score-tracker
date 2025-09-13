export type ClientMessage =
  | {
    CreateRoom: {
      admin_pass: string;
      code: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    JoinRoom: {
      name: string;
      room_code: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    LeaveRoom: RoomCode;
  }
  | {
    RemoveFromRoom: {
      id: number;
      room_code: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    DeleteRoom: RoomCode;
  }
  | {
    RequestAdmin: {
      password: string;
      room: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    BlessScore: {
      amount: number;
      to: number;
      [k: string]: unknown;
    };
  }
  | {
    RemoveScore: {
      amount: number;
      from: number;
      [k: string]: unknown;
    };
  }
  | {
    GiveScore: {
      amount: number;
      to: number;
      [k: string]: unknown;
    };
  }
  | {
    TransferScore: {
      amount: number;
      from: number;
      to: number;
      [k: string]: unknown;
    };
  }
  | {
    CreatePot: {
      description: string;
      room_code: RoomCode;
      score_requirement: number;
      [k: string]: unknown;
    };
  }
  | {
    JoinPot: {
      pot_id: number;
      room_code: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    ResolvePot: {
      pot_id: number;
      room_id: RoomCode;
      winner: number;
      [k: string]: unknown;
    };
  }
  | {
    CreateWager: {
      name: string;
      outcomes: WagerOutcome[];
      room_id: RoomCode;
      [k: string]: unknown;
    };
  }
  | {
    JoinWager: {
      amount: number;
      outcome_id: number;
      room_id: RoomCode;
      wager_id: number;
      [k: string]: unknown;
    };
  }
  | {
    ResolveWager: {
      outcome_id: number;
      room_id: RoomCode;
      wager_id: number;
      [k: string]: unknown;
    };
  };

export type RoomCode = string;

export interface WagerOutcome {
  description: string;
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
  | ("RoomDeleted" | "AdminGranted")
  | {
    SynchronizeRoom: {
      members: MemberState[];
      pots: Pot[];
      wager: Wager[];
      [k: string]: unknown;
    };
  }
  | {
    UserJoined: {
      id: number;
      name: string;
      [k: string]: unknown;
    };
  }
  | {
    UserRemoved: number;
  }
  | {
    PotCreated: Pot;
  }
  | {
    PotJoined: {
      pot_id: number;
      user_id: number;
      [k: string]: unknown;
    };
  }
  | {
    PotResolved: number;
  }
  | {
    WagerCreated: Wager;
  }
  | {
    WagerJoined: {
      amount: number;
      outcome_id: number;
      user_id: number;
      wager_id: number;
      [k: string]: unknown;
    };
  }
  | {
    WagerResolved: number;
  }
  | {
    ScoreChanged: {
      new_amount: number;
      user_id: number;
      [k: string]: unknown;
    };
  }
  | {
    Error: {
      description: string;
      display_to_user: boolean;
      [k: string]: unknown;
    };
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
