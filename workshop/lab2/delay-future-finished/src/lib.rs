use std::future::Future;
use std::pin::Pin;
use std::sync::{
    mpsc::{channel, Sender},
    Arc, Mutex,
};
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
    tx: Option<Sender<()>>,
}

pub fn delay_for(duration: Duration) -> Delay {
    let (tx, rx) = channel();

    let delay = Delay {
        state: Arc::new(Mutex::new(State {
            complete: false,
            waker: None,
            tx: Some(tx),
        })),
    };

    thread::spawn({
        let delay = delay.clone();
        move || {
            // wait to be notified that we can start
            let _ = rx.recv().unwrap();

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

            if let Some(tx) = self.state.lock().unwrap().tx.take() {
                // start the timer
                tx.send(()).unwrap();
            }

            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures::executor;
    use std::time::Instant;

    #[test]
    fn waits_one_second() {
        let start = Instant::now();

        executor::block_on(async {
            let d1 = delay_for(Duration::from_secs(1));
            let d2 = delay_for(Duration::from_secs(1));
            d1.await;
            d2.await;
        });

        assert!(start.elapsed() >= Duration::from_secs(2));
    }
}
