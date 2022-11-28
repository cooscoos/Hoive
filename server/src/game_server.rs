//! `GameServer` is an actor that allows players to send messages
//! to other players in the same room. It:
//! - keeps a list of connected client sessions
//! - manages lists of available game rooms.
//!
//!  This module defines the GameServer actor, and also defines:
//! - structs called "messages" that the GameServer actor will respond to
//! - handlers that define how GameServer responds to each message.

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

use std::{
    collections::{HashMap, HashSet},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use crate::{api::deregister_user, models::GameState};
use hoive::{game::comps::Team, pmoore::endgame_msg};

/// `GameServer` manages chat / game rooms and tracks
/// which client sessions are connected.
#[derive(Debug)]
pub struct GameServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rooms: HashMap<String, HashSet<usize>>,
    rng: ThreadRng,
    visitor_count: Arc<AtomicUsize>,
    visitor_list: HashMap<usize, String>,
}

impl GameServer {
    pub fn new(visitor_count: Arc<AtomicUsize>) -> GameServer {
        // New GameServers only have one room called "main"
        let mut rooms = HashMap::new();
        rooms.insert("main".to_owned(), HashSet::new());

        GameServer {
            sessions: HashMap::new(),
            rooms,
            rng: rand::thread_rng(),
            visitor_count,
            visitor_list: HashMap::new(),
        }
    }
}

impl GameServer {
    /// Send message to all users in the room
    fn send_message(&self, message: &str, room: &str, skip_id: usize) {
        if let Some(sessions) = self.rooms.get(room) {
            for id in sessions {
                if *id != skip_id {
                    if let Some(addr) = self.sessions.get(id) {
                        addr.do_send(Message(message.to_owned()));
                    }
                }
            }
        }
    }
}

/// Make GameServer an actor
impl Actor for GameServer {
    type Context = Context<Self>;
}

/// Define message. Can be sent to a session.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

// -----------------------------------------------------------------------
// 1. BASIC MESSAGES ==========================================~~~~~
// connect, disconnect, and send text to other connected clients
// -----------------------------------------------------------------------

/// Connect: Create a new client session on the GameServer
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub name: Option<String>,
}

/// Connect Handler: Register new client session and assign it a unique id.
impl Handler<Connect> for GameServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // Register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // Client joins "main" room on connect
        self.rooms
            .entry("main".to_owned())
            .or_insert_with(HashSet::new)
            .insert(id);

        // Increment number of visitors
        self.visitor_count.fetch_add(1, Ordering::SeqCst);

        id
    }
}

/// Disconnect client session of given id. Can optionally give client's username.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
    pub name: Option<String>,
}

/// Disconnect Handler: do disconnect, and tell everyone who left if username is defined.
impl Handler<Disconnect> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mut rooms: Vec<String> = Vec::new();

        // Remove address
        if self.sessions.remove(&msg.id).is_some() {
            // Remove session from all rooms
            for (name, sessions) in &mut self.rooms {
                if sessions.remove(&msg.id) {
                    rooms.push(name.to_owned());
                }
            }
        }

        self.visitor_count.fetch_sub(1, Ordering::SeqCst);

        // If the client had a username, tell everyone else in the same room they disconnected.
        if msg.name.is_some() {
            let name = msg.name.unwrap();
            let disc_msg = format!("{} disconnected", name);
            for room in rooms {
                self.send_message(&disc_msg, &room, 0);

                // Tell the other player you quit if this isn't main lobby
                if !room.contains("main"){
                    self.send_message("//cmd;disconnected", &room, 0);

                }

            }
        }

        // Remove them from the visitor list
        self.visitor_list.remove(&msg.id);

        // Deregister them from the sql db
        let _result = deregister_user(&msg.id);
    }
}

/// Send message to specific room
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Client id
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Game room name / id
    pub room: String,
}

/// ClientMessage Handler.
impl Handler<ClientMessage> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        // Don't send to self.
        self.send_message(msg.msg.as_str(), &msg.room, msg.id);
        // Do send to self
        //self.send_message(msg.msg.as_str(), &msg.room, 0);
    }
}

// -----------------------------------------------------------------------
// 2. USER MESSAGES ==========================================~~~~~
// join a room, change username, and see who else is online
// -----------------------------------------------------------------------

/// Join a room, if room doesn't exist, then create new one
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    /// Client id
    pub id: usize,
    /// Username
    pub username: String,
    /// Room name
    pub room: String,
}

/// Join Handler: send disconnect message to old room, send join message to new room
impl Handler<Join> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join {
            id,
            room: name,
            username,
        } = msg;
        let mut rooms = Vec::new();

        // remove session from all rooms
        for (n, sessions) in &mut self.rooms {
            if sessions.remove(&id) {
                rooms.push(n.to_owned());
            }
        }

        // send message to other users
        for room in rooms {
            self.send_message(&format!("{} left this room.", &username), &room, 0);
        }

        self.rooms
            .entry(name.clone())
            .or_insert_with(HashSet::new)
            .insert(id);

        self.send_message(&format!("{} joined this room.", &username), &name, id);
    }
}

/// Update username of client with given id
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewName {
    pub name: String,
    pub id: usize,
}

/// NewName Handler: change a username
impl Handler<NewName> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: NewName, _: &mut Context<Self>) -> Self::Result {
        self.visitor_list.insert(msg.id, msg.name.to_owned());

        // Notify all users that the new person joined
        self.send_message(&format!("\x1b[33;2m{} joined.\x1b[0m", msg.name), "main", 0);
    }
}

// /// Who: Request a list of the usernames of all connected clients
// #[derive(Message)]
// #[rtype(String)]
// pub struct CountVisitors;

// /// CountVisitors: display how many visitors there are
// impl Handler<CountVisitors> for GameServer {
//     type Result = String;

//     fn handle(&mut self, _msg: CountVisitors, _: &mut Context<Self>) -> Self::Result {
//         format!("There are {:?} people online.\n", self.visitor_count)
//     }
// }

/// WhoIn: What user_ids / usernames are in this room?
#[derive(Message, Debug)]
#[rtype(String)]
pub struct WhoIn {
    pub room: String,
}

/// Connect Handler: Register new client session and assign it a unique id.
impl Handler<WhoIn> for GameServer {
    type Result = String;

    fn handle(&mut self, msg: WhoIn, _: &mut Context<Self>) -> Self::Result {
        let mut return_string = String::new();
        // Get a list of user_ids for the room of interest.
        if let Some(user_id_list) = self.rooms.get(&msg.room) {
            let visitor_list = user_id_list
                .iter()
                .map(|c| self.visitor_list.get_key_value(c).unwrap())
                .map(|(k, v)| (*k, v.to_owned()))
                .collect::<HashMap<usize, String>>();

            // Serialize to json and push
            let serialized = serde_json::to_string(&visitor_list).unwrap();
            return_string.push_str(&serialized);
        };

        return_string
    }
}

// -----------------------------------------------------------------------
// 3. IN-GAME MESSAGES ==========================================~~~~~
// start a new game, update client gamestate, tell users there was a winner
// -----------------------------------------------------------------------

/// Start a new game room for a game of Hoive.
#[derive(Message)]
#[rtype(result = "()")]
pub struct NewGame {
    /// id of the game room / session. This is shared by the GameServer and database
    pub session_id: String,
    /// initial GameState, including id of the player who goes first
    pub game_state: GameState,
}

impl Handler<NewGame> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: NewGame, _: &mut Context<Self>) -> Self::Result {
        // Convert gamestate into text
        let gamestate_txt = serde_json::to_string(&msg.game_state).unwrap();

        // Notify all users to start a new game and send the gamestate
        self.send_message(
            &format!("//cmd;newgame;{}", gamestate_txt),
            &msg.session_id,
            0,
        );

        self.send_message(
            &format!("//cmd;room;{}", msg.session_id),
            &msg.session_id,
            0,
        );
    }
}

/// Notify all players in a given game session_id of an updated GameState
#[derive(Message)]
#[rtype(result = "()")]
pub struct UpdateGame {
    pub session_id: String,
    pub game_state: GameState,
}

/// UpdateGame Handler: tell all clients in a room to update their local GameState
impl Handler<UpdateGame> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: UpdateGame, _: &mut Context<Self>) -> Self::Result {
        // Convert gamestate into text
        let gamestate_txt = serde_json::to_string(&msg.game_state).unwrap();

        // Notify all users to update their gamestate
        self.send_message(
            &format!("//cmd;gamestate;{}", gamestate_txt),
            &msg.session_id,
            0,
        );
    }
}

/// Tell all players in a room that the game has been won by <username> on <team>
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct Winner {
    pub team: Option<Team>,
    pub room: String,
    pub username: Option<String>,
    /// Was the victory because other player forfeit?
    pub forfeit: bool,
}

/// Winner Handler: Tell all users in a game room who won and why
impl Handler<Winner> for GameServer {
    type Result = ();

    fn handle(&mut self, msg: Winner, _: &mut Context<Self>) {
        // Generate an endgame msg
        let mut endgame_msg = endgame_msg(msg.username.unwrap(), msg.team, msg.forfeit);
        endgame_msg.push_str("Rejoining main lobby...");

        // Send it out to everyone in the game room
        self.send_message(&endgame_msg, &msg.room, 0);

        // Reset their clients
        self.send_message("//cmd;goback", &msg.room, 0);

        // Create a message / handler to retrieve user names / ids of all players in  room.

        // // boot players. Need to figure out how to grab their usernames.
        // let usr1 = game_state.clone().user_1.unwrap();
        // let usr2 = game_state.user_2.unwrap();

        // gamesess.addr.do_send(game_server::Join {
        //     id: usr1.parse::<usize>().unwrap(),
        //     room: "main".to_string(),
        //     username: "player".to_string(),
        // });

        // gamesess.addr.do_send(game_server::Join {
        //     id: usr2.parse::<usize>().unwrap(),
        //     room: "main".to_string(),
        //     username: "player".to_string(),
        // });

        // // Deregister game from sql db
        // let _result = api::deregister_game(&session_id);
    }
}
