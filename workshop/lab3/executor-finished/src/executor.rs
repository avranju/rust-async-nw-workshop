use std::future::Future;
use std::sync::{
    mpsc::{sync_channel, Receiver, SyncSender},
    Arc, Mutex,
};
use std::task::{Context, Poll};

use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
};

pub fn new_runtime() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    (Executor { ready_queue }, Spawner { task_sender })
}

pub struct Task {
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    task_sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self
            .task_sender
            .send(arc_self.clone())
            .expect("too many tasks");
    }
}

#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

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
