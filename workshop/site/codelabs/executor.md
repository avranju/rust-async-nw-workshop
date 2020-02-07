Summary: Build an executor
Id: build-an-executor
Categories: rust
Status: Published
Feedback Link: https://github.com/avranju/rust-async-nw-workshop/issues
Authors: Rajasekharan Vengalil

# Build an executor

## What is an Executor?
Duration: 1

An executor is responsible for driving a future to completion. It is
typically handed an instance of a type that implements `Future` -- which
might be the root of a sub-graph of futures -- and is expected to drive
the entire graph of futures till all of them have returned `Poll::Ready`
from their `poll` functions.

### Create project

Create a new library project like so:

```
cargo new --lib executor
```

## Channels
Duration: 2

We'll use Rust [channels](https://doc.rust-lang.org/stable/std/sync/mpsc/fn.channel.html)
to co-ordinate work. Rust channels are an abstraction that allow 2 threads to safely
pass messages between them. A channel has a sender half and a receiving half. Both the
halves are connected to each other so that if one is destroyed then the other stops
functioning -- in other words, calls to send/receive messages will fail.

Channels can be bounded or unbounded. Bounded channels can be used to model backpressure.
The channel in the standard library is a multiple-producer-single-consumer implementation.
You can clone the sender half as many times as you like but you can have only one
receiver.

Here's an example from the standard library documentation:

```rust
use std::thread;
use std::sync::mpsc::channel;

// Create a simple streaming channel
let (tx, rx) = channel();

thread::spawn(move|| {
  tx.send(10).unwrap();
});

assert_eq!(rx.recv().unwrap(), 10);
```

## Task
Duration: 2

We'll define a `Task` type that represents a future that needs to run. The task
will include a `SyncSender` (the sender half of a bounded channel) that can be
used to re-schedule the future in case it is not complete yet.

```rust
pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}
```

`BoxFuture` is a type we will use from the [futures](https://crates.io/crates/futures) crate. A `BoxFuture` is a
type alias for a `Pin<Box<dyn Future<T>>>`.

## Waker
Duration: 15

In order for the executor to poll a future it'll need to provide the future with
a `Context`. A `Context` can be constructed from a `Waker` via the `Context::from_waker`
function.

Implementing a waker by hand requires careful construction of a [table of function pointers](https://doc.rust-lang.org/stable/std/task/struct.RawWakerVTable.html)
using unsafe code. The [futures](https://crates.io/crates/futures) crate
however provides a helper trait called [ArcWake](https://docs.rs/futures/0.3.1/futures/task/trait.ArcWake.html)
and a helper function called [waker_ref](https://docs.rs/futures/0.3.1/futures/task/fn.waker_ref.html)
which can be used to create a waker using only safe Rust. `waker_ref` provides a waker given a reference to any type `T`
that implements `ArcWake`.

`ArcWake` has been defined like so:

```rust
pub trait ArcWake: Send + Sync {
  fn wake_by_ref(arc_self: &Arc<Self>);

  fn wake(self: Arc<Self>) { ... }
}
```

In other words, if you have some type `T` that's wrapped in an `Arc` then `waker_ref`
is able to provider a `Waker` implementation in terms of the `ArcWake` implementation.

In the case of our `Task` struct we can simply clone the task and send it down the
channel via the `task_sender` field.

```rust
impl ArcWake for Task {
  fn wake_by_ref(arc_self: &Arc<Self>) {
    arc_self
      .task_sender
      .send(arc_self.clone())
      .expect("too many tasks");
  }
}
```

## Executor
Duration: 15

An executor will simply have a `Receiver` (the receive half of a bounded channel)
that it uses as the source of tasks to be run. Its basically a loop that keeps running
as long as the receiver is connected to at least one sender. When a task shows up
on the channel, it attempts to poll the future in the task and if it returns
`Poll::Pending` then it puts the future back into the task in order to poll again
when the future wakes the executor up later on. If the future returns `Poll::Ready`
then the executor simply drops the future.

```rust
pub struct Executor {
  ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
  pub fn run(&self) {
    while let Ok(task) = self.ready_queue.recv() {
      let mut future_slot = task.future.lock().unwrap();
      if let Some(mut future) = future_slot.take() {
        let waker = waker_ref(&task);
        let mut context = Context::from_waker(&waker);

        if let Poll::Pending = future.as_mut().poll(&mut context) {
          *future_slot = Some(future);
        }
      }
    }
  }
}
```

## Spawner
Duration: 15

The spawner is responsible for queueing new tasks with the executor. The spawner
will have the sender half of the channel.

```rust
#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}
```

The spawner will have a generic member function called `spawn` which takes as input
some type `T` that implements `Future<Output = ()>`. The `spawn` function must save
the future into a `Task` and send the task down the `task_sender` channel.

```
impl Spawner {
  pub fn spawn(&self, f: impl Future<Output = ()> + 'static + Send) {
    let f = Box::pin(f);
    let task = Arc::new(Task {
      future: Mutex::new(Some(f)),
      task_sender: self.task_sender.clone(),
    });

    self.task_sender.send(task).expect("too many tasks");
  }
}
```

## Delay timer
Duration: 3

Add the `Delay` timer implementation from the lab where you hand coded a future
into the executor project. Add it in a separate file called `delay.rs`:

```rust
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::Duration;

#[derive(Clone)]
pub struct Delay {
  state: Arc<Mutex<State>>,
}

#[derive(Clone)]
struct State {
  complete: bool,
  waker: Option<Waker>,
}

pub fn delay_for(duration: Duration) -> Delay {
  let delay = Delay {
    state: Arc::new(Mutex::new(State {
      complete: false,
      waker: None,
    })),
  };
  thread::spawn({
    let delay = delay.clone();
    move || {
      thread::sleep(duration);
      delay.state.lock().unwrap().complete = true;
      if let Some(waker) = delay.state.lock().unwrap().waker.take() {
        waker.wake();
      }
    }
  });

  delay
}

impl Future for Delay {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
    if self.state.lock().unwrap().complete {
      Poll::Ready(())
    } else {
      self.state.lock().unwrap().waker = Some(cx.waker().clone());
      Poll::Pending
    }
  }
}

```

## Tying it all together
Duration: 15

Add a helper function called `new_runtime` with the following signature so that it
is easy to instantiate an executor and a spawner together:

```rust
pub fn new_runtime() -> (Executor, Spawner) {
  todo!()
}
```

With all this in place, add the following test to your library project and try to
get it to pass!

```rust
#[cfg(test)]
mod tests {
  use super::*;

  use std::time::{Duration, Instant};

  use delay::delay_for;

  #[test]
  fn executor_works() {
    let (executor, spawner) = new_runtime();
    spawner.spawn(async move {
      delay_for(Duration::from_secs(1)).await;
    });

    drop(spawner);

    let start = Instant::now();
    executor.run();
    assert!(start.elapsed() >= Duration::from_secs(1));
  }
}
```