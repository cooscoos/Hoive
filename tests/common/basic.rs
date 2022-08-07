// These basic tests work for any triaxial co-ordinate system
use hoive::game::comps::Team;
use hoive::game::{board::Board, movestatus::MoveStatus};
use hoive::maths::coord::Coord;

pub fn first_turn<T: Coord>(board: &mut Board<T>) {
    // Place spider s1 at any position on the first turn and it should be fine
    let move_status = board.move_chip("a1", Team::Black, T::new(0, 0, 0));
    assert_eq!(MoveStatus::Success, move_status);
}

pub fn occupied<T: Coord>(board: &mut Board<T>) {
    // Try place a new chip on top of an existing one (illegal)
    board.move_chip("a1", Team::Black, T::new(0, 0, 0));
    let move_status = board.move_chip("a2", Team::Black, T::new(0, 0, 0));
    assert_eq!(MoveStatus::Occupied, move_status);
}

pub fn to_the_moon<T: Coord>(board: &mut Board<T>) {
    // Try place a new chip far away from all other chips (illegal)
    board.move_chip("a1", Team::Black, T::new(0, 0, 0));
    assert_eq!(
        MoveStatus::Unconnected,
        board.move_chip("a2", Team::Black, T::new(0, 0, 8))
    );
}
