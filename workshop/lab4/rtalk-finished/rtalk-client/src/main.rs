#![recursion_limit = "256"]

use std::env;
use std::error::Error;

use futures::select;
use futures_util::{future::FutureExt, sink::SinkExt};
use tokio::io;
use tokio::net::TcpStream;
use tokio::stream::StreamExt;
use tokio_util::codec::{Decoder, FramedRead, LinesCodec};

use rtalk_codec::{Event, EventCodec};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        println!("Usage: rtalk-client <user_name>");
        return Ok(());
    }
    let user_name = args[1].clone();

    let socket = TcpStream::connect("127.0.0.1:3215").await?;
    let codec = EventCodec;
    let mut framed = codec.framed(socket);

    // send a join message
    framed
        .send(Event::Join(user_name))
        .await
        .expect("Message send failed.");

    let mut stdin = FramedRead::new(io::stdin(), LinesCodec::new());
    let mut user_id = 0;

    loop {
        select! {
            event = framed.next().fuse() => {
                if let Some(Ok(event)) = event {
                    println!("{:?}", event);
                    match event {
                        Event::JoinResponse(id) => {
                            user_id = id;
                        }
                        Event::Message(_, msg) => {
                            println!("{}", msg);
                        }
                        _ => unreachable!(),
                    }
                }
            },
            msg = stdin.next().fuse() => {
                if let Some(Ok(msg)) = msg {
                    println!("You typed: {}", msg);
                    framed.send(Event::Message(user_id, msg)).await.expect("Message send failed.");
                    println!("Sent message event");
                }
            },
            complete => break,
        }
    }

    Ok(())
}
