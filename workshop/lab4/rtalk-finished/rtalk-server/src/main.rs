use std::collections::BTreeMap;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use futures::future;
use futures_util::sink::SinkExt;
use log::info;
use tokio::net::{TcpListener, TcpStream};
use tokio::stream::StreamExt;
use tokio_util::codec::{Decoder, Framed};

use rtalk_codec::{Event, EventCodec};

pub struct User {
    id: u64,
    name: String,
    framed: Pin<Box<Framed<TcpStream, EventCodec>>>,
}

struct State {
    counter: u64,
    users: BTreeMap<u64, User>,
}

impl State {
    fn add_user(&mut self, framed: Pin<Box<Framed<TcpStream, EventCodec>>>) -> u64 {
        self.counter += 1;

        self.users.insert(
            self.counter,
            User {
                id: self.counter,
                name: "".to_string(),
                framed,
            },
        );

        self.counter
    }

    fn update_user(&mut self, id: u64, name: String) {
        let user = self.users.get_mut(&id).unwrap();
        user.name = name;
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
        self.state.write().unwrap().add_user(framed)
    }

    fn update_user(&self, id: u64, name: String) {
        self.state.write().unwrap().update_user(id, name);
    }

    fn remove_user(&self, id: u64) {
        self.state.write().unwrap().users.remove(&id);
    }

    fn user_ids(&self) -> Vec<u64> {
        self.state
            .read()
            .unwrap()
            .users
            .iter()
            .map(|(_, u)| u.id)
            .collect()
    }

    async fn next_event(&self, id: u64) -> Option<Event> {
        let mut user = {
            let mut state = self.state.write().unwrap();
            state.users.remove(&id)?
        };

        let event = user.framed.as_mut().next().await?.ok();

        self.state.write().unwrap().users.insert(id, user);

        event
    }

    async fn send_event(&self, id: u64, evt: Event) {
        let mut user = {
            let mut state = self.state.write().unwrap();
            state.users.remove(&id).expect("User not found")
        };

        user.framed
            .as_mut()
            .send(evt)
            .await
            .expect("Couldn't write to client");

        self.state.write().unwrap().users.insert(id, user);
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
        let id = session.add_user(framed);

        tokio::spawn(async move {
            while let Some(event) = session.next_event(id).await {
                info!("{:?}", event);
                match event {
                    Event::Join(name) => {
                        session.update_user(id, name);
                        session.send_event(id, Event::JoinResponse(id)).await;
                    }
                    Event::Leave(id) => {
                        session.remove_user(id);
                    }
                    Event::Message(id, msg) => {
                        let futs = session.user_ids().into_iter().filter(|id2| *id2 != id).map(
                            |dest_id| session.send_event(dest_id, Event::Message(id, msg.clone())),
                        );
                        future::join_all(futs).await;
                    }
                    _ => unreachable!(),
                }
            }
        });
    }
}
