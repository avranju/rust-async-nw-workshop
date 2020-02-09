#![recursion_limit="512"]

use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures::{future,select};
use futures_util::{future::FutureExt,sink::SinkExt};
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_util::codec::{Decoder, Framed};

use rtalk_codec::{Event, EventCodec};

pub struct User {
    name: Option<String>,
    sender: Sender<Event>,
    task: tokio::task::JoinHandle<()>,
}

struct State {
    counter: u64,
    users: BTreeMap<u64, User>,
}

impl State {
    fn add_user(&mut self, session: Session, mut network: Pin<Box<Framed<TcpStream, EventCodec>>>) -> u64 {
        self.counter += 1;

        let id = self.counter;

        let (mut sender, mut rx) = mpsc::channel::<Event>(100);

        let task = tokio::spawn(async move {
            loop {
                select! {

                    // from session to network
                    event = rx.next().fuse() => {
                        if let Some(event) = event {
                            network.send(event).await.expect("Message send failed.");
                        }
                    },

                    // from network
                    event = network.next().fuse() => {
                        if let Some(Ok(event)) = event {
                            match event {
                                Event::RequestJoin(name) => {
                                    session.update_user(id, name.clone());
            
                                    let users = session.user_ids();
                                    let futs = users.into_iter().map(|dest_id| {
                                        session.send_event(dest_id, Event::Joined(name.clone()))
                                    });
                                    future::join_all(futs).await;
                                }
                                Event::Leave() => {
                                    let name = session.remove_user(id);
            
                                    let futs = session
                                        .user_ids()
                                        .into_iter()
                                        .map(|dest_id| session.send_event(dest_id, Event::Left(name.clone())));
                                    future::join_all(futs).await;
                                }
                                Event::MessageSend(msg) => {
                                    let who = session.get_name(id);
                                    let futs = session
                                        .user_ids()
                                        .into_iter()
                                        //.filter(|id2| *id2 != id)
                                        .map(|dest_id| {
                                            session.send_event(
                                                dest_id,
                                                Event::MessageReceived(who.clone(), msg.clone()),
                                            )
                                        });
                                    future::join_all(futs).await;
                                }
                                _ => unimplemented!()
                            }
                        }
                    }
                    complete => break,
                }
            }
        });

        self.users.insert(
            self.counter,
            User {
                name: None,
                sender,
                task,
            },
        );

        self.counter
    }

    fn get_name(&self, id: u64) -> Option<String> {
        let user = self.users.get(&id).unwrap();
        user.name.as_ref().map(|n| n.clone())
    }

    fn update_user(&mut self, id: u64, name: String) {
        let user = self.users.get_mut(&id).unwrap();
        user.name = Some(name);
    }
}

#[derive(Clone)]
pub struct Session {
    state: Arc<RwLock<State>>,
}

impl Session {
    fn new() -> Self {
        Session {
            state: Arc::new(RwLock::new(State {
                counter: 0,
                users: BTreeMap::new(),
            })),
        }
    }

    fn add_user(&self, framed: Pin<Box<Framed<TcpStream, EventCodec>>>) -> u64 {
        
        self.state.write().unwrap().add_user(self.clone(), framed)
    }

    fn get_name(&self, id: u64) -> String {
        self.state
            .read()
            .unwrap()
            .get_name(id)
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn update_user(&self, id: u64, name: String) {
        self.state.write().unwrap().update_user(id, name);
    }

    fn remove_user(&self, id: u64) -> String {
        let user = self.state.write().unwrap().users.remove(&id).unwrap();
        user.name.unwrap_or_else(|| "unknown".to_string())
    }

    fn user_ids(&self) -> Vec<u64> {
        self.state
            .read()
            .unwrap()
            .users
            .iter()
            .map(|(id, _)| *id)
            .collect()
    }

    async fn send_event(&self, id: u64, evt: Event) {
        let mut sender = {
            let state = self.state.read().unwrap();
            if let Some(user) = state.users.get(&id) {
                user.sender.clone()
            } else {
                return;
            }
        };

        sender.send(evt).await.expect("Could not queue event to send");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let session = Session::new();

    let mut listener = TcpListener::bind("127.0.0.1:3215").await?;
    loop {
        let (socket, _) = listener.accept().await?;

        let session = session.clone();
        let codec = EventCodec;
        let framed = Box::pin(codec.framed(socket));
        session.add_user(framed);
    }
}
