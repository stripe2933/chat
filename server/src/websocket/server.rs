//! `ChatServer` is an actor. It maintains list of connection client session.
//! And manages available conversations. Peers send messages to other peers in same
//! conversation through `ChatServer`.

use std::collections::{HashMap, HashSet};

use actix::prelude::*;
use rand::{self, rngs::ThreadRng, Rng};

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

/// Message for chat server communications

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: usize,
}

/// Send message to specific conversation
#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    /// Id of the client session
    pub id: usize,
    /// Peer message
    pub msg: String,
    /// Conversation name
    pub conversation: i64,
}

/// Join conversation, if conversation does not exists create new one.
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: usize, // client ID
    pub conversation_id: i64,
}

/// `ChatServer` manages chat conversations and responsible for coordinating chat session.
///
/// Implementation is very naïve.
#[derive(Debug)]
pub struct ChatServer {
    sessions: HashMap<usize, Recipient<Message>>,
    conversations: HashMap<i64, HashSet<usize>>,
    rng: ThreadRng,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        // default conversation
        let mut conversations = HashMap::new();
        conversations.insert(0, HashSet::new());

        ChatServer {
            sessions: HashMap::new(),
            conversations,
            rng: rand::thread_rng(),
        }
    }
}

impl ChatServer {
    /// Send message to all users in the conversation
    fn send_message(&self, conversation_id: i64, message: &str, skip_id: usize) {
        if let Some(sessions) = self.conversations.get(&conversation_id) {
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

/// Make actor from `ChatServer`
impl Actor for ChatServer {
    /// We are going to use simple Context, we just need ability to communicate
    /// with other actors.
    type Context = Context<Self>;
}

/// Handler for Connect message.
///
/// Register new session and assign unique id to this session
impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify all users in same conversation
        // self.send_message(0, "Someone joined", 0);

        // register session with random id
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        // auto join session to main conversation
        self.conversations.entry(0).or_default().insert(id);

        // send id back
        id
    }
}

/// Handler for Disconnect message.
impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        let mut conversation_ids = vec![];

        // remove address
        if self.sessions.remove(&msg.id).is_some() {
            // remove session from all conversations
            for (conversation_id, sessions) in &mut self.conversations {
                if sessions.remove(&msg.id) {
                    conversation_ids.push(*conversation_id);
                }
            }
        }

        // send message to other users
        // for conversation_id in conversation_ids {
        //     self.send_message(conversation_id, "Someone disconnected", 0);
        // }
    }
}

/// Handler for Message message.
impl Handler<ClientMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) {
        self.send_message(msg.conversation, msg.msg.as_str(), msg.id);
    }
}

/// Join conversation, send disconnect message to old conversation
/// send join message to new conversation
impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Context<Self>) {
        let Join { id, conversation_id } = msg;
        let mut conversations = Vec::new();

        // remove session from all conversations
        for (n, sessions) in &mut self.conversations {
            if sessions.remove(&id) {
                conversations.push(n.to_owned());
            }
        }

        // send message to other users
        // for conversation in conversations {
        //     self.send_message(conversation, "Someone disconnected", 0);
        // }

        self.conversations.entry(conversation_id).or_default().insert(id);
        // self.send_message(conversation_id, "Someone connected", id);
    }
}
