#![allow(dead_code)]

use futures::{
    future::{BoxFuture, FutureExt},
    task::{waker_ref, ArcWake},
};

use std::{
    future::Future,
    pin::Pin,
    process::Child,
    process::Command,
    sync::mpsc::{sync_channel, Receiver, SyncSender},
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
struct Task {
    // Why do we need a static lifetime here?
    // The mutex makes Rust happy. We can send / sync a mutex, but not the underlying BoxFuture.
    future: Mutex<BoxFuture<'static, ()>>,
    // We use an Arc because we can easily implement a waker for the arc, and not for the task.
    sender: SyncSender<Arc<Task>>,
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        arc_self.sender.send(arc_self.clone()).unwrap();
    }
}

pub struct Daemon {
    queue: Receiver<Arc<Task>>,
    sender: SyncSender<Arc<Task>>,
}

impl Daemon {
    pub fn new() -> Daemon {
        let (sender, queue) = sync_channel(100);
        Daemon { sender, queue }
    }

    // We can only box a future that can be sent.
    // We also add a static trait bound: https://doc.rust-lang.org/rust-by-example/scope/lifetime/static_lifetime.html#trait-bound
    pub fn enable(&self, future: impl Future<Output = ()> + 'static + Send) {
        let future = future.boxed();
        self.sender
            .send(Arc::new(Task {
                future: Mutex::new(future),
                sender: self.sender.clone(),
            }))
            .unwrap()
    }

    pub fn run(&self) {
        while let Ok(task) = self.queue.recv() {
            let waker = waker_ref(&task);
            let mut context = Context::from_waker(&waker);
            let mut fut = task.future.lock().unwrap();
            let _ = fut.as_mut().poll(&mut context);
        }
    }
}
