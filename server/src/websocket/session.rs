use std::time::{Duration, Instant};

use actix::prelude::*;
use actix_web::web;
use actix_web_actors::ws;
use serde::Serialize;

use crate::{websocket::server, AppState};

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsChatSession {
    pub id: usize, // Unique session id
    pub hb: Instant, // Client must send ping at least once per 10 seconds (CLIENT_TIMEOUT), otherwise we drop connection.
    pub conversation_id: i64, // Joined conversation
    pub username: String, // Peer username

    /// Websocket chat server
    // pub server_address: Addr<server::ChatServer>,
    pub app_state: web::Data<AppState>
}

#[derive(Serialize)]
enum WebsocketResponse{
    JoinStatus { success: bool },
    // Message { message: crate::api::message::Message },
    InvalidRequest,
}

impl WsChatSession {
    /// helper method that sends ping to client every 5 seconds (HEARTBEAT_INTERVAL).
    ///
    /// also this method checks heartbeats from client
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // check client heartbeats
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                // heartbeat timed out
                println!("Websocket Client heartbeat failed, disconnecting!");

                // notify chat server
                act.app_state.websocket_server.do_send(server::Disconnect { id: act.id });

                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }

            ctx.ping(b"");
        });
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;

    /// Method is called on actor start.
    /// We register ws session with ChatServer
    fn started(&mut self, ctx: &mut Self::Context) {
        // we'll start heartbeat process on session start.
        self.hb(ctx);

        // register self in chat server. `AsyncContext::wait` register
        // future within context, but context waits until this future resolves
        // before processing any other events.
        // HttpContext::state() is instance of WsChatSessionState, state is shared
        // across all routes within application
        let addr = ctx.address();
        self.app_state.websocket_server
            .send(server::Connect {
                addr: addr.recipient(),
            })
            .into_actor(self)
            .then(|res, act, ctx| {
                match res {
                    Ok(res) => act.id = res,
                    // something is wrong with chat server
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopping(&mut self, _: &mut Self::Context) -> Running {
        // notify chat server
        self.app_state.websocket_server.do_send(server::Disconnect { id: self.id });
        Running::Stop
    }
}

/// Handle messages from chat server, we simply send it to peer websocket
impl Handler<server::Message> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: server::Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

/// WebSocket message handler
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let AppState { database: _, websocket_server } = self.app_state.get_ref();

        let msg = match msg {
            Ok(msg) => msg,
            Err(_) => {
                ctx.stop();
                return;
            }
        };

        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
        match msg {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                // let text = text.trim();

                // we check for /sss type of messages
                if text.starts_with('/') {
                    let v: Vec<&str> = text.splitn(2, ' ').collect();
                    match v[..]{
                        ["/join", conversation_id] => {
                            let conversation_id = conversation_id.parse::<i64>().unwrap();

                            // Check if user joined to the given conversation.
                            // let is_user_joined_fut = async {
                            //      sqlx::query("SELECT 1 FROM group_members WHERE username = ? AND conversation_id = ?;")
                            //         .bind(self.username)
                            //         .bind(&conversation_id)
                            //         .fetch_optional(database)
                            //         .await.unwrap()
                            //         .is_some()
                            // };
                            // let is_user_joined_fut = actix::fut::wrap_future(is_user_joined_fut)
                            //     .map(|result, actor, ctx: &mut Self::Context| { 
                            //         if result{
                            //         }
                            //         else{
                            //             ctx.text("You are not joined to this conversation.");
                            //             ctx.stop();
                            //         }
                            //     });
                            // ctx.spawn(is_user_joined_fut);

                            self.conversation_id = conversation_id;
                            websocket_server.do_send(server::Join {
                                id: self.id,
                                conversation_id
                            });

                            ctx.text(serde_json::to_string(&WebsocketResponse::JoinStatus { success: true }).unwrap());
                        }
                        _ => ctx.text(serde_json::to_string(&WebsocketResponse::InvalidRequest).unwrap()),
                    }
                } else { // Message received.
                    // let username = self.username.clone();
                    // let text = text.clone().to_string();
                    
                    // let future = async move { 
                    //     sqlx::query!("INSERT INTO messages (sender_username, text, sent_at) VALUES (?, ?, datetime('now'));", username, text)
                    //         .execute(database);
                    // };  
                    // let future = actix::fut::wrap_future(future);
                    // ctx.wait(future);
                        // .await
                        // .unwrap();
                        
                    // send message to chat server
                    websocket_server.do_send(server::ClientMessage {
                        id: self.id,
                        msg: text.to_string(),
                        conversation: self.conversation_id,
                    })
                }
            }
            ws::Message::Close(reason) => {
                ctx.close(reason);
                ctx.stop();
            }
            ws::Message::Continuation(_) => {
                ctx.stop();
            }
            _ => (),
        }
    }
}
