use hoive::{coord, Board, MoveStatus, Team};

// cargo test hecs -- should pass for hecs co-ordinates

#[test]
fn first_turn() {
    // Place spider s1 at any position on the first turn and it should be fine
    let mut board = Board::default(coord::Hecs);
    let move_status = board.try_move("s1", Team::Black, (1, 0, 0));
    
    assert_eq!(MoveStatus::Success, move_status);
}

#[test]
fn hecs_basic_turn_occupied() {
    // Try place a new chip on top of an existing one (illegal)
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (0, 0, 0));
    assert_eq!(
        MoveStatus::Occupied,
        board.try_move("s2", Team::Black, (0, 0, 0))
    );
}

#[test]
fn hecs_basic_to_the_moon() {
    // Try place a new chip very far away from all other chips (illegal)
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (0, 0, 0));
    assert_eq!(
        MoveStatus::Unconnected,
        board.try_move("s2", Team::Black, (0, 0, 8))
    );
}

// These tests are hecs specific: cargo test hecs
#[test]
fn hecs_second_turn_neighbour() {
    // Place a white chip next to a black chip but on the second turn (should be okay)
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    assert_eq!(
        MoveStatus::Success,
        board.try_move("s1", Team::White, (0, 1, 0))
    );
}

#[test]
fn hecs_third_turn_badneighbour() {
    // Place a white chip next to a black chip on the third turn (that's illegal)
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    board.try_move("s1", Team::White, (0, 1, 0));
    assert_eq!(
        MoveStatus::BadNeighbour,
        board.try_move("s2", Team::White, (1, 0, 1))
    );
}

#[test]
fn hecs_fifth_turn_badneighbour() {
    // Do a bunch of legal stuff with a BadNeighbour move at the end
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    board.try_move("s1", Team::White, (0, 1, 0));
    board.try_move("s2", Team::Black, (0, 0, 0));
    board.try_move("s2", Team::White, (1, 1, 0));

    assert_eq!(
        MoveStatus::BadNeighbour,
        board.try_move("s3", Team::Black, (1, 1, 1))
    );
}

#[test]
fn hecs_split_hive() {
    // Put down four chips and then split the hive by moving a white spider from the middle
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    board.try_move("s1", Team::White, (0, 1, 0));
    board.try_move("s2", Team::Black, (0, 0, 0));
    board.try_move("s2", Team::White, (1, 1, 0));

    assert_eq!(
        MoveStatus::HiveSplit,
        board.try_move("s1", Team::White, (1, 1, 1))
    );
}

#[test]
fn hecs_nosplit_hive() {
    // Put down lots of chips and then do a move that doesn't split hive and is legal
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    board.try_move("s1", Team::White, (0, 1, 0));
    board.try_move("s2", Team::Black, (0, 0, 0));
    board.try_move("s2", Team::White, (1, 1, 0));
    board.try_move("s3", Team::White, (1, 1, -1));
    board.try_move("s4", Team::White, (0, 2, 0));

    assert_eq!(
        MoveStatus::Success,
        board.try_move("s3", Team::White, (1, 2, 0))
    );
}

#[test]
fn hecs_attack() {
    // Put down lots of chips and then relocate a white next to black after turn 6
    // We haven't coded logic for bee allowing move yet, so we'll need to rewrite this test then
    let mut board = Board::default(coord::Hecs);
    board.try_move("s1", Team::Black, (1, 0, 0));
    board.try_move("s1", Team::White, (0, 1, 0));
    board.try_move("s2", Team::Black, (0, 0, 0));
    board.try_move("s2", Team::White, (1, 1, 0));
    board.try_move("s3", Team::White, (1, 1, -1));
    board.try_move("s4", Team::White, (0, 2, 0));

    assert_eq!(
        MoveStatus::Success,
        board.try_move("s3", Team::White, (0, 0, 1))
    );
}
