/// Patrick Moore is the GamesMaster. He:
/// - provides a human-readable interface between players and the game logic;
/// - orchestrates normal/special moves in a way that tries to comply with game rules.
/// Pmoore's functions are used by clients.
use crate::draw::chipteam_to_str;
use crate::game::comps::{convert_static_basic, Chip, Team};
use crate::game::{actions::BoardAction, ask::Req, board::Board, movestatus::MoveStatus, specials};
use crate::maths::coord::{Coord, DoubleHeight};
use std::collections::BTreeSet;
use std::error::Error;

/// Say hello to the player
pub fn welcome() {
    println!(
        "
░█░█░█▀█░▀█▀░█░█░█▀▀
░█▀█░█░█░░█░░▀▄▀░█▀▀
░▀░▀░▀▀▀░▀▀▀░░▀░░▀▀▀

The boardgame Hive, in Rust.
"
    );
}

/// Decode a special string into a series of mosquito and/or pillbug actions
pub fn decode_specials<T: Coord>(
    board: &mut Board<T>,
    special: &str,
    active_team: Team,
    mut chip_name: &'static str,
    d_dest: DoubleHeight,
) -> MoveStatus {
    // Separate out the special's instructions using commas
    let items = special.split(',').collect::<Vec<&str>>();

    for (i, item) in items.clone().into_iter().enumerate() {
        // If we come across an m or a p, we need to read in the next 2 items to find col/row of victim
        if item == "m" || item == "p" {
            // Parse the victim coordinates into the board's coordinates
            let d_vic = DoubleHeight::from((
                items[i + 1].parse::<i8>().unwrap(),
                items[i + 2].parse::<i8>().unwrap(),
            ));
            let vic_coord = d_vic.mapto(board.coord);

            match item {
                "m" => {
                    // Get the mosquito's current position and ask it to absorb power from the victim
                    let position = board.get_position_byname(active_team, "m1").unwrap();
                    let newname = match specials::mosquito_suck(board, vic_coord, position) {
                        Some(value) => value,
                        None => return MoveStatus::NoSuck,
                    };
                    // Change the mosquito's name
                    chip_name = newname;
                }
                "p" => {
                    // Get the sumo-ing chip's position, parse destination and do the sumo
                    let position = board.get_position_byname(active_team, chip_name).unwrap();
                    let dest = d_dest.mapto(board.coord);
                    return specials::pillbug_sumo(board, vic_coord, dest, position);
                }
                _ => (), // ignore other entries
            }
        }
    }

    // if we get to this point without returning anything then we must be moving a mosquito, so do so
    board.move_chip(chip_name, active_team, d_dest.mapto(board.coord))
}

/// Try skip turn
pub fn skip_turn(action: &mut BoardAction) {
    action.special = Some("skip".to_string());
    action.request = Req::Execute;
}

/// Forfeit
pub fn forfeit(action: &mut BoardAction) {
    action.special = Some("forfeit".to_string());
    action.request = Req::Execute;
}

/// Use input string (textin) to select a chip from a active_team. Update the action.
pub fn select_chip_prompts<T: Coord>(
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
    active_team: Team,
) -> Result<(), Box<dyn Error>> {
    // The text input should define a chip to select
    let chip_select = match textin {
        _ if textin == "mb" => {
            // The player is probably trying to select their mosquito acting like a beetle
            convert_static_basic("m1".to_string())
        }
        _ if textin.contains('*') => {
            // The player is probably trying to select a beetle (or a mosquito acting like one).
            // Grab the first 2 chars of the string
            let (mut first, _) = textin.split_at(2);

            // If the first two chars are mosquito, convert to m1
            if first.contains('m') {
                first = "m1";
            }
            convert_static_basic(first.to_string())
        }
        _ if textin.starts_with(|c| c == 'l' || c == 'p' || c == 'q' || c == 'm') => {
            // Player can select ladybird, pillbug, queen, and mosquito without specifying the 1
            let proper_str = match textin.chars().next().unwrap() {
                'l' => "l1",
                'p' => "p1",
                'q' => "q1",
                'm' => "m1",
                _ => unreachable!(),
            };
            convert_static_basic(proper_str.to_string())
        }
        c => {
            // Otherwise, try and match a chip based on the input name
            convert_static_basic(c.to_owned())
        }
    };

    // Now, based on the chip selected
    match chip_select {
        None => {
            // Player tried to select a chip that doesn't exist.
            action.message =
                "You don't have this tile in your hand. Try select a chip again.".to_string();
        }
        Some(chip_name) => {
            // Start building the action with some default params.
            action.chip_name = chip_name.to_string();
            action.message = format!("Select co-ordinate to move {} to. Input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.", chipteam_to_str(chip_name,active_team));
            action.request = Req::Move;

            // Is the chip on the board, and can it possibly do a special move (e.g. sumo, mosquito suck)?
            let on_board = board.get_position_byname(active_team, chip_name);
            let can_special = on_board.is_some() && on_board.unwrap().get_layer() == 0;

            match chip_name {
                _ if (chip_name == "p1" || chip_name == "m1") && can_special => {
                    // Player selected pillbug / mosquito on the board

                    // Get pillbug / mosquito's current position, save to rowcol
                    let position = board.get_position_byname(active_team, chip_name).unwrap();
                    action.rowcol = Some(position.to_doubleheight(position));

                    // Get its neighbours
                    let neighbours = board.get_neighbour_chips(position);

                    // stick them into a BTree to preserve order.
                    let neighbours = neighbours.into_iter().collect::<BTreeSet<Chip>>();

                    // Update the boardaction
                    match chip_name {
                        "p1" => {
                            action.message =
                                "Hit m to sumo a neighbour, or anything else to do move."
                                    .to_string();
                            action.request = Req::Pillbug;
                        }
                        "m1" => {
                            action.message = format!(
                                "Select a neighbour to suck from...\n{}",
                                crate::draw::list_these_chips(neighbours.clone())
                            );
                            action.request = Req::Mosquito;
                        }
                        _ => unreachable!(),
                    }
                    action.neighbours = Some(neighbours);

                    // need to map to upper/lowercase string
                    // let neighbours = neighbours
                    //     .into_iter()
                    //     .map(|c| c.to_string())
                    //     .collect::<BTreeSet<String>>();
                    // Store the neighbours for later
                }
                _ => {} // nothing needs changing
            }
        }
    }

    Ok(())
}

/// Parse user inputs into a destination hex
pub fn move_chip_prompts(action: &mut BoardAction, textin: &str) -> Result<(), Box<dyn Error>> {
    //attempt to parse a move
    let usr_hex = crate::pmoore::coord_from_string(textin.to_owned());

    if let [Some(x), Some(y)] = usr_hex[..] {
        if (x + y) % 2 == 0 {
            action.rowcol = Some(DoubleHeight::from((x, y)));
            action.message = "Press enter to execute move on the game board".to_string();
            action.request = Req::Execute;
        }
    } else {
        action.message =
            "Invalid co-ordinates, enter coordinates again or hit x to abort.".to_string();
    }

    Ok(())
}

/// Converts an input number str (textin) into a mosquito action for sucking
pub fn mosquito_prompts<T: Coord>(
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
) -> Result<(), Box<dyn Error>> {
    // Choose a victim
    let victim_chip = match choose_victim_prompts(action, textin)? {
        Some(value) => value,
        None => return Ok(()),
    };

    // Get the position in doubleheight coordinates
    let victim_pos = board.get_dheight_position(&victim_chip)?;

    // Add to the action's special string to signify mosquito sucking victim at row,col
    let special = format!("m,{},{},", victim_pos.col, victim_pos.row);
    action.special = Some(special);

    if victim_chip.name != "p1" {
        action.message = format!(
            "You've absorbed from chip {}. Enter coordinates of where you would like to move to.",
            chipteam_to_str(victim_chip.name, victim_chip.team)
        );
        action.request = Req::Move;
    } else {
        // Player absorbed from a pillbug
        action.message = format!(
            "You've absorbed from chip {}. Hit m to sumo a neighbour, or enter coordinates of where you would like to move to.",
            chipteam_to_str(victim_chip.name, victim_chip.team)
        );
        action.request = Req::Pillbug;
    }

    Ok(())
}

/// Gives the player the option of sumo-ing a neighbour by hitting m key, or moving normally
pub fn pillbug_prompts(action: &mut BoardAction, textin: &str) -> Result<(), Box<dyn Error>> {
    match textin == "m" {
        true => {
            action.message = format!(
                "Select a neighbour to sumo from...\n{}",
                crate::draw::list_these_chips(action.neighbours.clone().unwrap())
            );
            action.request = Req::Sumo;
        }
        false => {
            action.message = "Select co-ordinate to move to. Input column then row, separated by comma, e.g.: 0, 0. Hit x to abort the move.".to_string();
            action.request = Req::Move;
        }
    }
    Ok(())
}

/// Allows user to select a victim chip to sumo with pillbug
pub fn sumo_victim_prompts<T: Coord>(
    action: &mut BoardAction,
    textin: &str,
    board: &Board<T>,
) -> Result<(), Box<dyn Error>> {
    // Choose a victim
    let victim_chip = match choose_victim_prompts(action, textin)? {
        Some(value) => value,
        None => return Ok(()),
    };

    // Get the position in doubleheight coordinates
    let victim_pos = board.get_dheight_position(&victim_chip)?;

    // Add to the action's special string to signify sumo victim at row,col
    let special = format!("p,{},{},", victim_pos.col, victim_pos.row);
    action.special = Some(special);
    action.message = format!("Select a co-ordinate to sumo chip {} to. Input column then row, separated by a comma, e.g.: 0, 0. Hit x to abort the sumo.",chipteam_to_str(victim_chip.name, victim_chip.team));

    action.request = Req::Move;

    Ok(())
}

/// Runs the user through selecting a neighbour to do a special move on. Returns the victim chip.
fn choose_victim_prompts(
    action: &mut BoardAction,
    textin: &str,
) -> Result<Option<Chip>, Box<dyn Error>> {
    let selection = match textin.parse::<usize>() {
        Ok(value) => value,
        Err(_) => {
            action.message = "That's not a valid number. Try again.".to_string();
            return Ok(None);
        }
    };

    let neighbours = action.neighbours.as_ref().unwrap();

    if selection > neighbours.len() - 1 {
        action.message = "Pick a number from the options. Try again.".to_string();
        return Ok(None);
    }

    let selected = neighbours
        .iter()
        .nth(selection)
        .expect("Problem selecting chip.");

    // Get the coordinates of that selected chip
    let victim_chip = Chip {
        name: convert_static_basic(selected.name.to_owned()).expect("Invalid chip"),
        team: selected.team, //get_team_from_chip(&selected.team),
    };

    Ok(Some(victim_chip))
}

/// Parse comma separated values input by a user to a doubleheight co-ordinate
pub fn coord_from_string(str: String) -> Vec<Option<i8>> {
    str.trim()
        .split(',')
        .map(|c| match c.parse::<i8>() {
            Ok(value) => Some(value),
            Err(_) => None,
        })
        .collect::<Vec<Option<i8>>>()
}

/// ~~~ OooOoooh ~~~
pub fn xylophone() -> &'static str {
    "                                     \n    ....,,..                        \n    .......',;cllllooll:'.                     \n  ..;cccccodxxkkkOkkkxxxol;.                   \n .:ooooooddxkkkkOOOOOOOkxxdl,                  \n.cdddoooddxxkkkkOO0000OOOOkkx:.                \n'loddolooddxkkOOO00000000OOOO0x.                \n.;ldxdlllodxxkOO000KKKK0000OOO0x'                \n,codo::clodxkOO00KKKKKKKK00Okkk:                \n.,::;,;cldxkkOO00000KKK0000OkkOl.               \n.','.';:ldxxkOOO0000OO000O0OOkxo,               \n....',;:loxxkkkkOOkkOO00O0OOkxxd'              \n.....';cclodkkOkkkkOOO00OOOOkxxd'              \n .  ...,:looodkkkxxxkkkkkkkkxxxo.              \n .   .'';ldoodoolloddddddddoxxxo.              \n     ....,,',,;;::ldollccldxOkxo.              \n    .....'',::c::ox0OkdxxkOO0Oxl.              \n    ..'';:cllddc:lx0OOOOOO0Okxdl.              \n   ....';clcldl,;lx0OkxkOOOkdddc.              \n  ..   ..,cool'.;ld0Okdoddxxxxdl.              \n  .. ....':c:,...,:ldxkkdlodxxxo'              \n  .......',,,,....':dkOOkxdxxdxl.              \n  ......,::,.''';:coooddddddddd,.              \n  .......,cc,',,,,;:ccldxdllll;.               \n............','.,;::cloodddoc:c:.                \n.;;;'..''........',::clcclllooxo'                .\n.oxddl,.','''.......''''',:ldOKO:                 .\n.d0000ko;',:c;.    .....,ldOKNWNd.                 .\n.oKKXKX0kkcco;. ......;:;,oXWWMNk;.                 .\n.o00KKXX0O00Od,...''cxKNNXkldXWWO:'.                  \n'd00000KXKKXKkc.....c0NWWMWNK0X0x:,;.                  \n,d000KKKXXXXOl.',. .oXNNWWMWXXXKd'':;.                  \n.:OKKXXXXXNX0l.  ....:KWNWWWWWNX0d;.;:,.                  \n.o0XKXXXXXXKo'      .;ONNNWWWWNXKx:.,;;'.                  \n,xXXXXXXXKxl'   ..   .dNNNNNWNXXKkc'',''.                   \nOXXXXXXXk, ...   .    ;kKXNNNNXXKx,......                   \n0KKKXNXKd.   ...     ..cdxk0KNNN0l. ..',.                   \nkkxkkO00Ol.       ...  .lkxddddl,....:k0l.                  \n..';ldxx0k, .      .'.  :0Ol'. ..'. ;OXXx."
}

/// Create fancy message to say player "name" on "team" won the game, and why.
pub fn endgame_msg(player_name: String, team: Option<Team>, forfeit: bool) -> String {
    let mut endgame_msg =
        "\x1b[33;1m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\x1b[0m\n\n".to_string();

    // Create message send to all users saying who won and why
    match team {
        Some(team) => {
            endgame_msg.push_str(&format!("{} team wins ", crate::draw::team_string(team)))
        }
        None => endgame_msg
            .push_str("It's a draw.\n\n\x1b[32;1m== Both teams are defeated == \x1b[0m\n\n"),
    };

    if team.is_some() {
        match forfeit {
            true => endgame_msg.push_str("because the other player forfeit!"),
            false => endgame_msg.push_str("by destroying opponent's queen bee!"),
        }
        endgame_msg.push_str(&format!(
            "\n\n\x1b[32;1m== {} wins the game == \x1b[0m\n\n",
            player_name
        ));
    }

    endgame_msg.push_str("\x1b[33;1m~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~\x1b[0m\n\n");

    endgame_msg
}

/// Returns the game manual.
pub fn help_me() -> &'static str {
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
Type x at any time to abort moves, or press enter to see the peices
on the board and in your hand.\n
You can attempt to move any peice you own that is in your hand
or on the board. The game won't provide hints about whether a move is
legal until the move is attempted.\n
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
