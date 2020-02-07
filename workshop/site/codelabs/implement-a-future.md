Summary: Implement a future by hand
Id: implement-a-future
Categories: rust
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Implement a future by hand

## Delay future
Duration: 1

The idea is to implement a future that does the work that Tokio's
[delay_for](https://docs.rs/tokio/0.2.11/tokio/time/fn.delay_for.html) function does.
`delay_for` takes as input a duration and returns a future that resolves when that
time duration has elapsed. Here's an [example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=6a6ac8facf27db3a5e172538697cd6e3)
of how this can be used:

```rust
use std::time::Duration;
use tokio::time::delay_for;

#[tokio::main]
async fn main() {
    println!("waiting 2 seconds");

    // wait 2 seconds
    delay_for(Duration::from_secs(2)).await;

    println!("Done waiting.");
}
```

Positive
: Try running the program given above in the [Rust playground](https://play.rust-lang.org/).

Implementing an efficient production grade timer is a very [involved topic](http://www.cs.columbia.edu/~nahum/w6998/papers/ton97-timing-wheels.pdf).
We are going to implement a simple-minded inefficient one that just spawns a thread that sleeps
for the desired duration.

## Creating threads
Duration: 2

Use the [std::thread::spawn](https://doc.rust-lang.org/stable/std/thread/fn.spawn.html) API to
create native threads. Here's a [small example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=d8d4c073e5cd6eef5e52c9b97631e9bd):

```rust
use std::thread;

fn main() {
    print("Starting thread.");
    let th = thread::spawn(|| {
        print("Thread runneth over.");
    });
    
    th.join().unwrap();
    print("All done.");
}

fn print(msg: &str) {
    println!("[{:?}] {}", thread::current().id(), msg);
}
```

When you run this, you should see output that looks like this:

```
[ThreadId(1)] Starting thread.
[ThreadId(2)] Thread runneth over.
[ThreadId(1)] All done.
```

## Sharing mutable state between threads
Duration: 3

Rust's standard library provides synchronization primitives that you can use when you need
to mutate shared state between threads. The [Arc](https://doc.rust-lang.org/stable/std/sync/struct.Arc.html)
type is a thread-safe reference counted smart pointer. The [Mutex](https://doc.rust-lang.org/stable/std/sync/struct.Mutex.html)
type, as you might imagine, allows you to get exclusive access to some shared data. Use these
together to safely share mutable state across threads.

Here's an [example](https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=2100db0fc215a2c89a801d1fb66a955f):

```rust
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Clone)]
struct Delay {
    state: Arc<Mutex<State>>,
}

impl Delay {
    fn new(complete: bool) -> Self {
        Delay {
            state: Arc::new(Mutex::new(State { complete }))
        }
    }
}

struct State {
    complete: bool,
}

fn main() {
    let d = Delay::new(false);
    
    thread::spawn({
        let d = d.clone();
        move || {
            d.state.lock().unwrap().complete = true;
        }
    });
    
    let t2 = thread::spawn({
        let d = d.clone();
        move || {
            while !d.state.lock().unwrap().complete {}
        }
    });
    
    t2.join().unwrap();
    println!("complete = {}", d.state.lock().unwrap().complete);
}
```

## Implement Delay future
Duration: 25

Create a new library project using `cargo`:

```
cargo new --lib delay-future
```

Your implementation might have the following general structure:

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

struct Delay {
    // stuff here
}

fn delay_for(duration: Duration) -> Delay {
    // create and return Delay
    todo!()
}

impl Future for Delay {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        todo!()
    }
}
```

The goal is to implement the future for `Delay` so that the following test passes:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor;
    use std::time::Instant;

    #[test]
    fn waits_one_second() {
        let start = Instant::now();

        executor::block_on(async {
            delay_for(Duration::from_secs(1)).await;
        });

        assert!(start.elapsed() >= Duration::from_secs(1));
    }
}
```