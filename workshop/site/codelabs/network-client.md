Summary: Write a network client
Id: network-client
Categories: rust
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Write a network client

## Your mission
Duration: 2

Your mission, should you choose to accept is to write a network client for an
existing server app. In the workshop git repo, navigate to `workshop/lab4/rtalk-start`
and open the folder in VS Code. You should find 3 crates in it:

- rtalk-codec
- rtalk-server
- rtalk-client

## rtalk-server
Duration: 30

This is a fairly simple chat server. It has the ability to accept TCP connections
from multiple clients. It maintains an in-memory session and assigns unique integral
identifiers for each user that connects. It receives messages and broadcasts them.
It listens for connections on TCP port `3215`.

Spend some time reviewing the server code and see how it is implemented. Identify
areas that you may be able to improve the implementation.

## rtalk-codec
Duration: 15

This crate implements the data transfer codec. Events sent on the wire are represented
in-memory as an enum:

```rust
pub enum Event {
    Join(String),
    JoinResponse(u64),
    Leave(u64),
    Message(u64, String),
}
```

This crate provides an implementation for the `Encoder` and `Decoder` traits from the
`tokio-util` crate. This implementation is common and can be reused from the client app.
Review this implementation and see if you can re-factor the code so it is structured
better.

## rtalk-client
Duration: 40

The `rtalk-client` crate currently is unimplemented. Your task is to review the server
implementation and try to come up with an implementation that does the following:

- Opens a TCP connection to the server on port `3215`
- Sends an `Event::Join` message as soon as it connects
- Starts up a loop that's listening for messages from the server and from standard
  input and does the appropriate processing, i.e., when an `Event::Message` is
  received from the server, print the message to the screen and when a line of
  text has been read from the terminal, send an `Event::Message` to the server.