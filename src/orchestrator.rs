use std::pin::Pin;
use std::{collections::HashMap, io};

use crate::newunit::ServiceDef;
use futures::channel::oneshot::Sender;
use futures::channel::{mpsc, oneshot};
use futures::future::Fuse;
use futures::future::FutureExt;
use futures::*;

use futures::stream::FuturesUnordered;
use futures::{
    future::FusedFuture,
    pin_mut, select,
    stream::{FusedStream, Stream, StreamExt},
};

pub struct ServiceState {
    definition: ServiceDef,
    started: bool,
    pub once_started: Vec<Sender<()>>,
    pub once_failed: Vec<Sender<()>>,
}

impl ServiceState {
    pub fn new(definition: ServiceDef) -> Self {
        Self {
            definition,
            started: false,
            once_started: vec![],
            once_failed: vec![],
        }
    }
}
pub struct Orchestrator {
    pub sender: mpsc::Sender<Message>,
    receiver: mpsc::Receiver<Message>,
    pub services: HashMap<String, ServiceState>,
}
pub enum Message {
    FailOn {
        service: String,
        sender: oneshot::Sender<()>,
    },
    DependOn {
        service: String,
        sender: oneshot::Sender<()>,
    },
    Start {
        service: String,
    },
    Started {
        service: String,
        result: io::Result<()>,
    },
    Finished {
        service: String,
        result: io::Result<()>,
    },
}

impl Orchestrator {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel(100);
        let services = HashMap::new();
        Self {
            sender,
            receiver,
            services,
        }
    }

    pub async fn run(&mut self) {
        let mut services = FuturesUnordered::new();
        loop {
            select! {
            maybe_message = self.receiver.next() => {
                match maybe_message {
                Some(message) => {
                    let service = self.receive(message);
                    services.push(service);
                }
                None => ()
                }
            },
            () = services.select_next_some() => {

            }
            };
        }
    }

    fn receive(&mut self, message: Message) -> Pin<Box<dyn Future<Output = ()>>> {
        use Message::*;
        match message {
            Start { service } => {
                println!("Service {} is starting...", service);
                self.services.entry(service.clone()).and_modify(|state| {
                    state.started = true;
                });
                let definition = self
                    .services
                    .get(&service)
                    .expect("Service should exist.")
                    .definition
                    .clone();
                let sender = self.sender.clone();
                let future = async move {
                    let receivers = definition.clone().cancel_if_failed(sender.clone()).await;
                    let () = definition.clone().start_wanted(sender.clone()).await;
                    let deps = definition
                        .clone()
                        .start_required_after(sender.clone())
                        .await;
                    if deps.is_ok() {
                        println!("Deps for service {} are ok", service);
                        // BUG: There is a problem with this select.
                        if receivers.is_empty() {
                            println!("No receivers for {}", service);
                        } else {
                            println!("Receivers for {}", service);
                        }
                        let mut fut = FuturesUnordered::from_iter(receivers);
                        let fut2 = definition.clone().run(sender.clone()).fuse();
                        pin_mut!(fut2);
                        select! {
                            x = fut.select_next_some() => {
                            println!("Cancelled {}!!!", service)
                            }
                            x = fut2 => {
                            println!("Finished {}", service)
                            }
                        }
                    } else {
                        println!("Cannot start {} due to a dependency failure", service)
                    }
                };
                Box::pin(future)
            }
            Started { service, result } => {
                println!("Service {} has started with {}", service, result.is_ok());
                let state = self
                    .services
                    .get_mut(&service)
                    .expect("Started service should exist.");
                if result.is_ok() {
                    while let Some(sender) = state.once_started.pop() {
                        // TODO: The receiver may have been dropped.
                        sender.send(()).unwrap();
                    }
                } else {
                    state.once_started.clear();
                }
                Box::pin(std::future::ready(()))
            }
            Finished { service, result } => {
                println!("Service {} has finished with {}", service, result.is_ok());
                let state = self
                    .services
                    .get_mut(&service)
                    .expect("Started service should exist.");
                if result.is_err() {
                    while let Some(sender) = state.once_failed.pop() {
                        // TODO: The receiver may have been dropped.
                        sender.send(()).unwrap();
                    }
                }
                Box::pin(std::future::ready(()))
            }
            DependOn { service, sender } => {
                println!("Service {} has a dependency", service);
                self.services.entry(service).and_modify(|state| {
                    state.once_started.push(sender);
                });
                Box::pin(std::future::ready(()))
            }
            FailOn { service, sender } => {
                println!("Service {} has a fail dependency", service);
                self.services.entry(service).and_modify(|state| {
                    state.once_failed.push(sender);
                });
                Box::pin(std::future::ready(()))
            }
        }
    }
}
