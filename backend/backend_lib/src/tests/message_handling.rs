/*
* Does each message sent respond with the appropriate messages?
*/

use rstest::{fixture, rstest};

use super::*;
use crate::{
    connection::message::{ClientMessage, Destination, ServerMessage},
    state::{
        ID,
        error::{AdminRequestError, PotMutationError, RoomMutationError, WagerMutationError},
        room::{
            MemberState,
            pot::Pot,
            wager::{Wager, WagerOutcome},
        },
    },
    tests::StateFixture,
};

use lazy_static::lazy_static;
type MessageTestExpected = Vec<(ServerMessage, Destination)>;
type MessageTestErrorExpected = Vec<MessageHandleError>;

lazy_static! {
    static ref ROOM_CODE_1: RoomCode = RoomCode::from("AAAAAAAA");
    static ref ROOM_CODE_2: RoomCode = RoomCode::from("BBBBBBBB");
    static ref USER_NAME_1: String = "user1".to_owned();
    static ref USER_ADDR_1: SocketAddr = "127.0.0.1:8080".parse().unwrap();
    static ref USER_NAME_2: String = "user2".to_owned();
    static ref USER_ADDR_2: SocketAddr = "127.0.0.1:8081".parse().unwrap();
    static ref SCORE_AMT_1: i64 = 100;
    static ref SCORE_AMT_2: i64 = 1000;
    static ref WAGER_1: Wager = Wager::new(
        0,
        "wager".to_owned(),
        vec![
            WagerOutcome::new("Outcome 1".to_owned(), "outcome 1".to_owned(), 30),
            WagerOutcome::new("Outcome 2".to_owned(), "outcome 2".to_owned(), 70)
        ]
    );
}

impl MockConnection {
    fn assert_success(&self, expected: MessageTestExpected) {
        assert!(
            self.errors.is_empty(),
            "\n unexpected errors: \n {}",
            self.errors.iter().map(|x| format!("{x:?}")).join("\n")
        );
        assert_eq!(
            self.recieved,
            expected,
            "\n recieved messages: \n {} \n != \n expected messages: \n {}",
            self.recieved.iter().map(|x| format!("{x:?}")).join("\n"),
            expected.iter().map(|x| format!("{x:?}")).join("\n")
        );
    }
    fn assert_failure(&self, expected: MessageTestErrorExpected) {
        assert!(
            self.recieved.is_empty(),
            "unexpected successes: \n {}",
            self.recieved.iter().map(|x| format!("{x:?}")).join("\n")
        );
        assert_eq!(
            self.errors,
            expected,
            "\n Recieved errors: \n {} \n != \n Expected Errors: \n {}",
            self.errors.iter().map(|x| format!("{x:?}")).join("\n"),
            expected.iter().map(|x| format!("{x:?}")).join("\n")
        )
    }
}

#[fixture]
fn room_creation_expected() -> MessageTestExpected {
    Vec::new()
}
#[fixture]
fn room_creation_invalid_expected() -> MessageTestErrorExpected {
    vec![MessageHandleError::RoomAlreadyExists(*ROOM_CODE_1)]
}
#[fixture]
fn room_joining_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::UserJoined {
                name: USER_NAME_1.clone(),
                id: 0,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::SynchronizeRoom {
                members: vec![MemberState::new(USER_NAME_1.clone(), 0)],
                pots: vec![],
                wager: vec![],
            },
            Destination::Myself,
        ),
    ]
}
#[fixture]
fn room_joining_invalid_expected() -> MessageTestErrorExpected {
    vec![MessageHandleError::NonexistentRoom(*ROOM_CODE_1)]
}
#[fixture]
fn leaving_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::UserRemoved { id: 0 },
        Destination::PeersExclusive,
    )]
}
#[fixture]
fn leaving_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        MessageHandleError::RoomMutationError(RoomMutationError::AddressNotInRoom(
            *USER_ADDR_1,
            *ROOM_CODE_1,
        )),
    ]
}
#[fixture]
fn removing_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::UserRemoved { id: 1 },
        Destination::PeersInclusive,
    )]
}
#[fixture]
fn removing_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        MessageHandleError::RoomMutationError(RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1)),
    ]
}
#[fixture]
fn deletion_expected() -> MessageTestExpected {
    vec![(ServerMessage::RoomDeleted, Destination::PeersInclusive)]
}
#[fixture]
fn deletion_invalid_expected() -> MessageTestErrorExpected {
    vec![MessageHandleError::NonexistentRoom(*ROOM_CODE_1)]
}
#[fixture]
fn request_success_expected() -> MessageTestExpected {
    vec![(ServerMessage::AdminGranted, Destination::Myself)]
}
#[fixture]
fn request_failure_expected() -> MessageTestErrorExpected {
    vec![
        (MessageHandleError::NonexistentRoom(*ROOM_CODE_1)),
        RoomMutationError::AddressNotInRoom(*USER_ADDR_1, *ROOM_CODE_1).into(),
        AdminRequestError::IncorrectPassword.into(),
        AdminRequestError::AlreadyAdmin.into(),
    ]
}
#[fixture]
fn blessing_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::ScoreChanged {
            user_id: 0,
            new_amount: *SCORE_AMT_1,
        },
        Destination::PeersInclusive,
    )]
}
#[fixture]
fn blessing_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::UserNotInAnyRoom(*USER_ADDR_1),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
        RoomMutationError::NegativeScore.into(),
    ]
}
#[fixture]
fn score_removal_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::ScoreChanged {
            user_id: 0,
            new_amount: *SCORE_AMT_2 - *SCORE_AMT_1,
        },
        Destination::PeersInclusive,
    )]
}
#[fixture]
fn score_removal_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::UserNotInAnyRoom(*USER_ADDR_1),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
        RoomMutationError::NegativeScore.into(),
    ]
}
#[fixture]
fn score_giving_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: *SCORE_AMT_2 - *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::ScoreChanged {
                user_id: 1,
                new_amount: *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn score_giving_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::UserNotInAnyRoom(*USER_ADDR_1),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
        RoomMutationError::NegativeScore.into(),
    ]
}
#[fixture]
fn score_transferal_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: *SCORE_AMT_2 - *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::ScoreChanged {
                user_id: 1,
                new_amount: *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn score_transferal_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::UserNotInAnyRoom(*USER_ADDR_1),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
        RoomMutationError::UserNotInRoom(2, *ROOM_CODE_1).into(),
        RoomMutationError::NegativeScore.into(),
    ]
}
#[fixture]
fn wager_creation_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::WagerCreated {
            wager: WAGER_1.clone(),
        },
        Destination::PeersInclusive,
    )]
}
#[fixture]
fn wager_creation_invalid_expected() -> MessageTestErrorExpected {
    vec![MessageHandleError::NonexistentRoom(*ROOM_CODE_1)]
}
#[fixture]
fn wager_joining_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::WagerJoined {
                wager_id: 0,
                user_id: 0,
                outcome_id: 0,
                amount: *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: *SCORE_AMT_2 - *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn wager_joining_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        RoomMutationError::AddressNotInRoom(*USER_ADDR_1, *ROOM_CODE_1).into(),
        RoomMutationError::NonexistentWager {
            wager_id: 0,
            room_code: *ROOM_CODE_1,
        }
        .into(),
        RoomMutationError::NegativeScore.into(),
        WagerMutationError::UserAlreadyExists {
            user_id: 0,
            wager_id: 0,
        }
        .into(),
    ]
}

//let score_diff: i64 = (((*bet as f128) * score_mult).round() as i64) + bet;
#[fixture]
fn wager_resolution_expected() -> MessageTestExpected {
    let score_diff: i64 =
        (((*SCORE_AMT_1 as f128) * (30 as f128 / 100.0)).round() as i64) + *SCORE_AMT_1;
    vec![
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: score_diff,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::WagerResolved { id: 0 },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn wager_resolution_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        RoomMutationError::NonexistentWager {
            wager_id: 0,
            room_code: *ROOM_CODE_1,
        }
        .into(),
        WagerMutationError::NonexistentOutcome {
            outcome_id: 2,
            wager_id: 0,
        }
        .into(),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
    ]
}
#[fixture]
fn pot_creation_expected() -> MessageTestExpected {
    vec![(
        ServerMessage::PotCreated {
            pot: Pot::new(0, *SCORE_AMT_1, "description".to_owned()),
        },
        Destination::PeersInclusive,
    )]
}
#[fixture]
fn pot_creation_invalid_expected() -> MessageTestErrorExpected {
    vec![MessageHandleError::NonexistentRoom(*ROOM_CODE_1)]
}
#[fixture]
fn pot_joining_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::PotJoined {
                pot_id: 0,
                user_id: 0,
            },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: *SCORE_AMT_2 - *SCORE_AMT_1,
            },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn pot_joining_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        RoomMutationError::AddressNotInRoom(*USER_ADDR_1, *ROOM_CODE_1).into(),
        RoomMutationError::NonexistentPot(0, *ROOM_CODE_1).into(),
        PotMutationError::InsufficientScore {
            user_id: 0,
            pot_id: 0,
            user_score: 0,
            score_req: *SCORE_AMT_1,
        }
        .into(),
    ]
}
#[fixture]
fn pot_resolution_expected() -> MessageTestExpected {
    vec![
        (
            ServerMessage::PotResolved { id: 0 },
            Destination::PeersInclusive,
        ),
        (
            ServerMessage::ScoreChanged {
                user_id: 0,
                new_amount: *SCORE_AMT_1 * 2,
            },
            Destination::PeersInclusive,
        ),
    ]
}
#[fixture]
fn pot_resolution_invalid_expected() -> MessageTestErrorExpected {
    vec![
        MessageHandleError::NonexistentRoom(*ROOM_CODE_1),
        RoomMutationError::NonexistentPot(0, *ROOM_CODE_1).into(),
        RoomMutationError::UserNotInRoom(1, *ROOM_CODE_1).into(),
    ]
}

#[rstest]
fn room_creation(multi_client_state: StateFixture, room_creation_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    connections[0].send_message(ClientMessage::CreateRoom {
        code: *ROOM_CODE_1,
        admin_pass: "pass".to_string(),
    });
    connections[0].assert_success(room_creation_expected);
}
#[rstest]
fn room_creation_invalid(
    multi_client_state: StateFixture,
    room_creation_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    connections[0].send_message(ClientMessage::CreateRoom {
        code: *ROOM_CODE_1,
        admin_pass: "pass".to_owned(),
    });
    connections[0].assert_failure(room_creation_invalid_expected);
}
#[rstest]
fn room_joining(multi_client_state: StateFixture, room_joining_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);

    connections[0].send_message(ClientMessage::JoinRoom {
        code: *ROOM_CODE_1,
        name: USER_NAME_1.clone(),
    });
    connections[0].assert_success(room_joining_expected);
}
#[rstest]
fn room_joining_invalid(
    multi_client_state: StateFixture,
    room_joining_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    connections[0].send_message(ClientMessage::JoinRoom {
        code: *ROOM_CODE_1,
        name: "grog".into(),
    });
    connections[0].assert_failure(room_joining_invalid_expected);
}
#[rstest]
fn room_leaving(multi_client_state: StateFixture, leaving_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup(&mut connections[0]);
    connections[0].send_message(ClientMessage::LeaveRoom {
        room_code: *ROOM_CODE_1,
    });
    connections[0].assert_success(leaving_expected);
}
#[rstest]
fn room_leaving_invalid(
    multi_client_state: StateFixture,
    leaving_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::LeaveRoom {
        room_code: *ROOM_CODE_1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(leaving_invalid_expected);
}
#[rstest]
fn room_removing(multi_client_state: StateFixture, removing_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    connections[0].send_message(ClientMessage::RemoveFromRoom {
        code: *ROOM_CODE_1,
        id: 1,
    });
    connections[0].assert_success(removing_expected);
}
#[rstest]
fn room_removing_invalid(
    multi_client_state: StateFixture,
    removing_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::RemoveFromRoom {
        code: *ROOM_CODE_1,
        id: 1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(removing_invalid_expected);
}
#[rstest]
fn room_deletion(multi_client_state: StateFixture, deletion_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(ClientMessage::DeleteRoom {
        room_code: *ROOM_CODE_1,
    });
    connections[0].assert_success(deletion_expected);
}
#[rstest]
fn room_deletion_invalid(
    multi_client_state: StateFixture,
    deletion_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    connections[0].send_message(ClientMessage::DeleteRoom {
        room_code: *ROOM_CODE_1,
    });
    connections[0].assert_failure(deletion_invalid_expected);
}
#[rstest]
fn admin_request_success(
    multi_client_state: StateFixture,
    request_success_expected: MessageTestExpected,
) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup(&mut connections[0]);
    connections[0].send_message(ClientMessage::RequestAdmin {
        room: *ROOM_CODE_1,
        password: "pass".to_owned(),
    });
    connections[0].assert_success(request_success_expected);
}
#[rstest]
fn admin_request_failure(
    multi_client_state: StateFixture,
    request_failure_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::RequestAdmin {
        room: *ROOM_CODE_1,
        password: "pass".to_owned(),
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[0]);
    connections[0].send_message(ClientMessage::RequestAdmin {
        room: *ROOM_CODE_1,
        password: "bad".into(),
    });
    connections[0].send_message_setup(msg.clone());
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(request_failure_expected);
}
#[rstest]
fn score_blessing(multi_client_state: StateFixture, blessing_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(ClientMessage::BlessScore {
        to: 0,
        amount: *SCORE_AMT_1,
    });
    connections[0].assert_success(blessing_expected);
}
#[rstest]
fn score_blessing_invalid(
    multi_client_state: StateFixture,
    blessing_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::BlessScore {
        to: 1,
        amount: *SCORE_AMT_1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[1]);
    connections[0].send_message(ClientMessage::BlessScore {
        to: 1,
        amount: -(*SCORE_AMT_1),
    });
    connections[0].assert_failure(blessing_invalid_expected);
}
#[rstest]
fn score_removal(multi_client_state: StateFixture, score_removal_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    connections[0].send_message(ClientMessage::RemoveScore {
        from: 0,
        amount: *SCORE_AMT_1,
    });
    connections[0].assert_success(score_removal_expected);
}
#[rstest]
fn score_removal_invalid(
    multi_client_state: StateFixture,
    score_removal_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::RemoveScore { from: 1, amount: 1 };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[1]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(score_removal_invalid_expected);
}
#[rstest]
fn score_giving(multi_client_state: StateFixture, score_giving_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    connections[0].send_message(ClientMessage::GiveScore {
        to: 1,
        amount: *SCORE_AMT_1,
    });
    connections[0].assert_success(score_giving_expected);
}
#[rstest]
fn score_giving_invalid(
    multi_client_state: StateFixture,
    score_giving_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::GiveScore {
        to: 1,
        amount: *SCORE_AMT_1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[1]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(score_giving_invalid_expected);
}
#[rstest]
fn score_transferal(
    multi_client_state: StateFixture,
    score_transferal_expected: MessageTestExpected,
) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    connections[0].send_message(ClientMessage::TransferScore {
        from: 0,
        to: 1,
        amount: *SCORE_AMT_1,
    });
    connections[0].assert_success(score_transferal_expected);
}
#[rstest]
fn score_transferal_invalid(
    multi_client_state: StateFixture,
    score_transferal_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::TransferScore {
        from: 2,
        to: 1,
        amount: *SCORE_AMT_1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[1]);
    connections[0].send_message(msg.clone());
    room_join_setup(&mut connections[2]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(score_transferal_invalid_expected);
}

#[rstest]
fn pot_creation(multi_client_state: StateFixture, pot_creation_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(ClientMessage::CreatePot {
        room_code: *ROOM_CODE_1,
        score_requirement: *SCORE_AMT_1,
        description: "description".to_owned(),
    });
    connections[0].assert_success(pot_creation_expected);
}
#[rstest]
fn pot_creation_invalid(
    multi_client_state: StateFixture,
    pot_creation_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    connections[0].send_message(ClientMessage::CreatePot {
        room_code: *ROOM_CODE_1,
        score_requirement: *SCORE_AMT_1,
        description: "description".to_owned(),
    });
    connections[0].assert_failure(pot_creation_invalid_expected);
}
#[rstest]
fn pot_joining(multi_client_state: StateFixture, pot_joining_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    room_setup_pot(&mut connections[0]);
    connections[0].send_message(ClientMessage::JoinPot {
        room_code: *ROOM_CODE_1,
        pot_id: 0,
    });
    connections[0].assert_success(pot_joining_expected);
}
#[rstest]
fn pot_joining_invalid(
    multi_client_state: StateFixture,
    pot_joining_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::JoinPot {
        room_code: *ROOM_CODE_1,
        pot_id: 0,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_setup_pot(&mut connections[0]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(pot_joining_invalid_expected);
}
#[rstest]
fn pot_resolution(multi_client_state: StateFixture, pot_resolution_expected: MessageTestExpected) {
    let (s, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    room_setup_score(&mut connections[0], [0, 1], *SCORE_AMT_1);
    room_setup_pot(&mut connections[0]);
    room_join_pot(&mut connections[0]);
    room_join_pot(&mut connections[1]);
    println!("{s:?}");
    connections[0].send_message(ClientMessage::ResolvePot {
        room_id: *ROOM_CODE_1,
        pot_id: 0,
        winner: 0,
    });
    connections[0].assert_success(pot_resolution_expected);
}
#[rstest]
fn pot_resolution_invalid(
    multi_client_state: StateFixture,
    pot_resolution_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::ResolvePot {
        room_id: *ROOM_CODE_1,
        pot_id: 0,
        winner: 1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_setup_pot(&mut connections[0]);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(pot_resolution_invalid_expected);
}
#[rstest]
fn wager_creation(multi_client_state: StateFixture, wager_creation_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(ClientMessage::CreateWager {
        room_id: *ROOM_CODE_1,
        name: WAGER_1.name(),
        outcomes: WAGER_1.outcomes(),
    });
    connections[0].assert_success(wager_creation_expected);
}
#[rstest]
fn wager_creation_invalid(
    multi_client_state: StateFixture,
    wager_creation_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    connections[0].send_message(ClientMessage::CreateWager {
        room_id: *ROOM_CODE_1,
        name: WAGER_1.name(),
        outcomes: WAGER_1.outcomes(),
    });
    connections[0].assert_failure(wager_creation_invalid_expected)
}
#[rstest]
fn wager_joining(multi_client_state: StateFixture, wager_joining_expected: MessageTestExpected) {
    let (_, mut connections) = multi_client_state;
    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_setup_wager(&mut connections[0]);
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    connections[0].send_message(ClientMessage::JoinWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id: 0,
        amount: *SCORE_AMT_1,
    });
    connections[0].assert_success(wager_joining_expected)
}

#[rstest]
fn wager_joining_invalid(
    multi_client_state: StateFixture,
    wager_joining_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::JoinWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id: 0,
        amount: *SCORE_AMT_1,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup_admin(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_setup_wager(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_setup_score(&mut connections[0], [0], *SCORE_AMT_2);
    connections[0].send_message(msg.clone());
    connections[0].assert_failure(wager_joining_invalid_expected);
}
#[rstest]
fn wager_resolution(
    multi_client_state: StateFixture,
    wager_resolution_expected: MessageTestExpected,
) {
    let (_, mut connections) = multi_client_state;

    room_init(&mut connections[0]);
    room_join_setup_admin(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    room_setup_score(&mut connections[0], [0, 1], *SCORE_AMT_1);
    room_setup_wager(&mut connections[0]);
    room_join_wager(&mut connections[0], 0);
    room_join_wager(&mut connections[1], 1);
    connections[0].send_message(ClientMessage::ResolveWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id: 0,
    });
    connections[0].assert_success(wager_resolution_expected);
}
#[rstest]
fn wager_resolution_invalid(
    multi_client_state: StateFixture,
    wager_resolution_invalid_expected: MessageTestErrorExpected,
) {
    let (_, mut connections) = multi_client_state;
    let msg = ClientMessage::ResolveWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id: 2,
    };
    connections[0].send_message(msg.clone());
    room_init(&mut connections[0]);
    connections[0].send_message(msg.clone());
    room_join_setup_admin(&mut connections[0]);
    room_join_setup(&mut connections[1]);
    room_setup_score(&mut connections[0], [0, 1], *SCORE_AMT_2);
    room_setup_wager(&mut connections[0]);
    room_join_wager(&mut connections[0], 0);
    room_join_wager(&mut connections[1], 0);
    connections[0].send_message(msg.clone());
    connections[1].send_message_setup(ClientMessage::LeaveRoom {
        room_code: *ROOM_CODE_1,
    });
    connections[0].send_message(ClientMessage::ResolveWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id: 0,
    });
    connections[0].assert_failure(wager_resolution_invalid_expected);
}

fn room_init(con: &mut MockConnection) {
    con.send_message_setup(ClientMessage::CreateRoom {
        code: *ROOM_CODE_1,
        admin_pass: "pass".to_owned(),
    });
}

fn room_join_setup(con: &mut MockConnection) {
    con.send_message_setup(ClientMessage::JoinRoom {
        code: *ROOM_CODE_1,
        name: USER_NAME_1.clone(),
    });
}

fn room_join_setup_admin(con: &mut MockConnection) {
    room_join_setup(con);
    con.send_message_setup(ClientMessage::RequestAdmin {
        room: *ROOM_CODE_1,
        password: "pass".to_owned(),
    });
}

fn room_setup_score(con: &mut MockConnection, ids: impl IntoIterator<Item = ID>, amt: i64) {
    for id in ids {
        con.send_message_setup(ClientMessage::BlessScore {
            to: id,
            amount: amt,
        });
    }
}

fn room_setup_pot(con: &mut MockConnection) {
    con.send_message_setup(ClientMessage::CreatePot {
        room_code: *ROOM_CODE_1,
        score_requirement: *SCORE_AMT_1,
        description: "description".to_owned(),
    });
}

fn room_join_pot(con: &mut MockConnection) {
    con.send_message_setup(ClientMessage::JoinPot {
        room_code: *ROOM_CODE_1,
        pot_id: 0,
    });
}

fn room_setup_wager(con: &mut MockConnection) {
    con.send_message_setup(ClientMessage::CreateWager {
        room_id: *ROOM_CODE_1,
        name: WAGER_1.name(),
        outcomes: WAGER_1.outcomes(),
    });
}

fn room_join_wager(con: &mut MockConnection, outcome_id: ID) {
    con.send_message_setup(ClientMessage::JoinWager {
        room_id: *ROOM_CODE_1,
        wager_id: 0,
        outcome_id,
        amount: *SCORE_AMT_1,
    });
}
