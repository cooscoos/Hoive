/// Take actions to play live games of Hoive on the server

use reqwest::Client;
use std::collections::BTreeSet;
use std::{error::Error, thread, time::Duration};

use crate::comms;
use server::models::{GameState, Winner};
use hoive::game::actions::BoardAction;

use hoive::{draw, pmoore::get_usr_input};
use hoive::game::comps::{convert_static_basic, Team, Chip};
use hoive::game::{board::Board, movestatus::MoveStatus, specials};
use hoive::maths::coord::{Coord, DoubleHeight};

/// Ask player to take a turn
pub async fn take_turn<T: Coord>(
    board: &Board<T>,
    active_team: Team,
    client: &Client,
    base_url: &String,
) -> Result<GameState, Box<dyn Error>> {
    println!("{}\n", draw::show_board(&board));
    'turn: loop {
        // Ask player to do action, provide them with response message, break loop if move was successful
        let temp_move_status = act(&mut board.clone(), active_team, &client, &base_url).await?;

        let move_status = match temp_move_status {
            MoveStatus::SkipTurn => comms::send_action(BoardAction::skip(), client, base_url).await?,
            MoveStatus::Forfeit => comms::send_action(BoardAction::forfeit(), client, base_url).await?,
            _ => temp_move_status,
        };

        println!("{}",move_status.to_string());
        if  move_status == MoveStatus::Success {
            break 'turn;
        }
    }

    // Update the local game state based on server db
    comms::get_gamestate(&client, &base_url).await
}

/// Poll the server every few seconds to check if other player is done with their move.
pub async fn observe<T: Coord>(
    board: &mut Board<T>,
    my_team: Team,
    client: &Client,
    base_url: &String,
) -> Result<GameState, Box<dyn Error>> {

    println!("{}\n", draw::show_board(&board));

    // Update the board based on info on the server
    let mut game_state = comms::get_gamestate(&client, &base_url).await?;

    println!("Waiting for other player to take turn...");

    // If the last person who took turn is you, then we're still waiting for other player
    while game_state.last_user_id.as_ref().unwrap() == &my_team.to_string() {
        // Wait a few seconds, refresh gamestate
        thread::sleep(Duration::from_secs(5));
        game_state = comms::get_gamestate(&client, &base_url).await?;
    }
    Ok(game_state)

}

/// Tell the player who won, ask them if they want to play again
pub fn endgame(winner: Winner, my_team: Team) -> bool {
    let mut endgame_msg = match winner.team {
        Some(team) if team == my_team => "You win ".to_string(),
        Some(team) if team != my_team => "You lose ".to_string(),
        None => "It's a draw!".to_string(),
        Some(_) => panic!("Unrecognised team has won"),
    };

    match winner.forfeit {
        true => endgame_msg.push_str("by forfeit!"),
        false => endgame_msg.push_str("by destruction of queen bee!"),
    }

    println!("{endgame_msg}");

    println!("Hit y to play again, anything else to quit.");
    get_usr_input() == "y"
}

/// For the team who are playing, take guided actions and request those actions from the server's db.
/// This client is as naive as possible to allow the server to do most of the work checking if moves
/// are legal. This keeps the client lightweight, and makes it harder to cheat.
pub async fn act<T: Coord>(
    board: &mut Board<T>,
    active_team: Team,
    client: &Client,
    base_url: &String,
) -> Result<MoveStatus, Box<dyn Error>> {

    println!("Team {}, it's your turn!", draw::team_string(active_team));

    // Keep asking player to select chip until Some(value) happens
    let mut chip_selection = None;
    while chip_selection == None {
        chip_selection = hoive::pmoore::chip_select(board, active_team)
    }

    // The user's entry decides what chip to select (or what to do next)
    // Safe to unwrap because of loop above
    let base_chip_name = match chip_selection.unwrap() {
        "w" => return Ok(MoveStatus::SkipTurn), // try and skip turn
        "quit" => return Ok(MoveStatus::Forfeit), // try and forfeit
        valid_name => valid_name,
    };


    // Check for mosquito and pillbugs and update chip names as required
    let (chip_name, mut special_string, textin, is_pillbug)= match hoive::pmoore::mosquito_pillbug_checks(board, base_chip_name,active_team) {
        Some(values) => values,
        None => return Ok(MoveStatus::Nothing),
    };

    // If the user hits m then try execute a pillbug's special move
    let action = if textin == "m" && is_pillbug {
        // Create a sumo special string to signify pillbug sumo
        let (victim_source, victim_dest) = match pillbug_prompts(board, chip_name, active_team) {
            Some(value) => value,
            None => return Ok(MoveStatus::Nothing)
        };

        // Generate a special string to signify pillbug tossing victim at row,col
        if !special_string.is_empty(){
            special_string.push_str(",")
        }

        special_string.push_str(&format!("p,{},{}",victim_source.col, victim_source.row));

        BoardAction::do_move(base_chip_name, active_team, victim_dest.col, victim_dest.row, special_string)
        
      
    } else if textin == "m" && !is_pillbug {
        println!("This chip doesn't have special moves!");
        return Ok(MoveStatus::Nothing)
    } else {

        match coord_prompts(textin) {
            // Otherwise try move the chip if the movement prompts are valid
            Some((row, col)) => {
                // Request an action and send it to server
                BoardAction::do_move(base_chip_name, active_team, row, col, special_string)
                // println!("Sending action {:?}", action);
                //comms::send_action(action, client, base_url).await
            }
            None => return Ok(MoveStatus::Nothing),
        }
    };
    let return_status = comms::send_action(action, client, base_url).await;
    return_status

    // later countdown timer for turn
}


/// Ask user to select a coordinate or hit enter to return None so that we can
/// abort the parent function.
fn coord_prompts(mut textin: String) -> Option<(i8, i8)> {
    if textin.is_empty() {
        return None;
    }; // escape this function and start again

    let usr_hex = coord_from_string(textin);

    match usr_hex[..] {
        [Some(x), Some(y)] => {
            match (x + y) % 2 {
                // The sum of doubleheight coords should always be an even no.
                0 => Some((x, y)),
                _ => {
                    println!("Invalid co-ordinates, try again. Enter to abort.");
                    textin = get_usr_input();
                    coord_prompts(textin)
                }
            }
        }
        _ => {
            println!("Try again: enter two numbers separated by a comma. Enter to abort.");
            textin = get_usr_input();
            coord_prompts(textin)
        }
    }
}


/// Leads the player through executing a pillbug's sumo special move.
fn pillbug_prompts<T: Coord>(
    board: &mut Board<T>,
    chip_name: &'static str,
    active_team: Team,
) -> Option<(DoubleHeight, DoubleHeight)> {
    // Get pillbug's position and prompt the user to select a neighbouring chip to sumo, returning the coords of the victim
    let position = board.get_position_byname(active_team, chip_name).unwrap();
    let source = match neighbour_prompts(board, position, "sumo".to_string()) {
        Some(value) => value,
        None => return None, // abort special move
    };

    // Ask player to select a co-ordinate to sumo to
    println!("Select a co-ordinate to sumo this chip to. Input column then row, separated by a comma, e.g.: 0, 0. Hit enter to abort the sumo.");
    let textin = get_usr_input();
    let coord = match coord_prompts(textin) {
        None => return None, // abort move
        Some((row, col)) => (row, col),
    };

    // Convert from doubleheight to the game's co-ordinate system
    let dest = board.coord.mapfrom_doubleheight(DoubleHeight::from(coord));

    // Try execute the move and show the game's messages.
    // Do this on the server
    // specials::pillbug_sumo(board, source, dest, position)

    // return the source, dest in dheight coords
    Some((source.to_doubleheight(source), dest.to_doubleheight(dest)))


}


/// Ask the player to select neighbouring chips from a list (will present colour-coded options 0-5)
fn neighbour_prompts<T: Coord>(board: &mut Board<T>, position: T, movename: String) -> Option<T> {
    let neighbours = board.get_neighbour_chips(position);

    // stick them into a BTree to preserve order.
    let neighbours = neighbours.into_iter().collect::<BTreeSet<Chip>>();

    // Ask player to select neighbouring chips from a list (presenting options 0-6 for white and black team chips)
    println!(
        "Select which chip to {} by entering a number up to {}. Hit enter to abort.\n {}",
        movename,
        neighbours.len() - 1,
        draw::list_these_chips(neighbours.clone())
    );

    let textin = get_usr_input();

    // Returning none will abort the special move
    if textin.is_empty() {
        return None;
    }

    // Match to the player's selection
    let selection = match textin.parse::<usize>() {
        Ok(value) if value < neighbours.len() + 1 => value,
        _ => {
            println!("Use a number from the list");
            return None;
        }
    };

    let selected = neighbours.into_iter().nth(selection).unwrap();

    // get the co-ordinate of the selected chip and return them
    let source = board.chips.get(&selected).unwrap().unwrap();
    Some(source)
}

/// Parse comma separated values input by a user to a doubleheight co-ordinate
fn coord_from_string(str: String) -> Vec<Option<i8>> {
    str.trim()
        .split(',')
        .map(|c| match c.parse::<i8>() {
            Ok(value) => Some(value),
            Err(_) => None,
        })
        .collect::<Vec<Option<i8>>>()
}

/// ~~~ OooOoooh ~~~
fn xylophone() {
    let egg = "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx.";
    println!("{egg}");
}

/// Returns the game manual.
fn help_me() -> &'static str {
    "
----------------------------------------------------------------\n
= How to play =\n
Each player starts the game with the following peices in their hand:\n
- 1 bee (q1 or q)
- 2 spiders (s1, s2)
- 3 ants (a1, a2, a3)
- 2 beetles (b1, b2)
- 3 grasshoppers (g1, g2, g3)
- a mosquito (m1 or m)
- a ladybird (l1 or l)
- a pill bug (p1 or p).\n
Select one of the peices above using the codes given in brackets,
and then enter a location to move the peice to on the board using
comma separated values e.g. 1,-3.\n
The dots in the terminal represent the centres of hexagons: positions
where each hexagonal peice can be placed. Each hex is surrounded by 6 neighbours.\n
Press enter at any time to abort moves, or to see the peices
on the board and in your hand.\n
You can attempt to move any peice you own that is in your hand
or on the board. The game won't hint about whether a move is
legal, but it will tell you if an attempted action is illegal.\n
Beetles can move on top of the hive. When they are on
top of the hive, they will be have an asterix (*) next
to their name. They can be reselected later by typing their code
with or without this asterix.\n
Pillbugs can sumo pieces next to them. To sumo, hit
the m key when prompted.\n
Mosquitos need to absorb the power of a neighbour before
they take any action.\n
If a mosquito absorbs power from a beetle and
ends up on top of the hive, it will be represented as
mb*. It can be selected using: m1, mb, or mb*.\n
Game rules: https://en.wikipedia.org/wiki/Hive_(game)\n
----------------------------------------------------------------
"
}

// Do we need to serialize? with json?
// #[derive(Serialize, Deserialize, Default, Debug, Clone)]
// pub struct User {
//     pub id: String,
// }
