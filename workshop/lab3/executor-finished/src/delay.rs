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
