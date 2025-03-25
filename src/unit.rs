#![allow(dead_code)]

use std::{
    future::Future,
    pin::Pin,
    process::Child,
    process::Command,
    task::{Context, Poll},
    time::Duration,
};

pub struct OneshotBuilder<'a> {
    name: String,
    prog: &'a str,
    args: Vec<&'a str>,
    schedule: Option<Duration>,
}
use futures::future::BoxFuture;
use futures::future::FutureExt;

impl<'a> OneshotBuilder<'a> {
    pub fn new(name: String, prog: &'a str, args: impl IntoIterator<Item = &'a str>) -> Self {
        Self {
            name,
            prog,
            args: args.into_iter().collect::<Vec<_>>(),
            schedule: None,
        }
    }

    pub fn schedule(&mut self, every: Duration) {
        self.schedule = Some(every)
    }
    pub fn build(self) {
        async {
            let mut command = Command::new(self.prog);
            command.args(self.args);
            Oneshot {
                name: self.name,
                command,
                process: None,
            }
            .await;
        }
        .boxed();
    }
}

pub struct Oneshot {
    name: String,
    command: Command,
    process: Option<Child>,
}

impl Future for Oneshot {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        match &mut state.process {
            None => {
                println!("Starting {}", state.name);
                let proc = state.command.spawn().unwrap();
                state.process = Some(proc);
                // Note that we call the waker each time. This is horribly inefficient.
                cx.waker().clone().wake();
                Poll::Pending
            }
            Some(proc) => {
                let result = proc.try_wait().unwrap();
                match result {
                    None => {
                        cx.waker().clone().wake();
                        Poll::Pending
                    }
                    Some(_) => {
                        println!("Finished {}", state.name);
                        Poll::Ready(())
                    }
                }
            }
        }
    }
}
use futures::channel::oneshot::Receiver;
use futures::channel::oneshot::Sender;
use futures::future::join_all;

pub struct Notify {
    name: String,
    command: Command,
    delay: Option<Duration>,
    senders: Vec<Sender<()>>,
    receivers: Vec<Receiver<()>>,
}

impl Notify {
    pub fn new<'a>(name: &str, prog: &str, args: impl IntoIterator<Item = &'a str>) -> Notify {
        let mut command = Command::new(prog);
        command.args(args);
        Notify {
            name: name.to_string(),
            command,
            delay: None,
            senders: vec![],
            receivers: vec![],
        }
    }

    pub fn to_oneshot(self) -> Oneshot {
        Oneshot {
            name: self.name,
            command: self.command,
            process: None,
        }
    }

    pub fn require(&mut self, other: &mut Notify) {
        let (send, recv) = futures::channel::oneshot::channel::<()>();
        self.receivers.push(recv);
        other.senders.push(send);
    }

    pub fn schedule(&mut self, after: Duration) {
        self.delay = Some(after);
    }

    pub async fn build(mut self) {
        let receivers = self.receivers.drain(0..).collect::<Vec<_>>();
        let senders = self.senders.drain(0..).collect::<Vec<_>>();
        futures::future::join_all(receivers).await;
        let delay = self.delay.take();
        if let Some(time) = delay {
            println!("Waiting for {:?}", time);
            async_std::task::sleep(time).await;
        }
        let res = self.await;
        senders.into_iter().for_each(|s| s.send(()).unwrap());
        res.await;
        println!("Finished end");
    }
}
impl Future for Notify {
    type Output = NotifyEnd;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        println!("Starting notify {}", state.name);
        let proc = state.command.spawn().unwrap();
        Poll::Ready(NotifyEnd {
            name: state.name.clone(),
            proc,
        })
    }
}
pub struct NotifyEnd {
    name: String,
    proc: Child,
}

impl Future for NotifyEnd {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let state = self.get_mut();
        let result = state.proc.try_wait().unwrap();
        match result {
            None => {
                cx.waker().clone().wake();
                Poll::Pending
            }
            Some(_) => {
                println!("Finished notify {}", state.name);
                Poll::Ready(())
            }
        }
    }
}
