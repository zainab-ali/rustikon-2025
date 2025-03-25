#![allow(dead_code)]

use std::collections::HashMap;
use std::io;

use async_std::stream::StreamExt;
use futures::channel::mpsc;
use futures::channel::oneshot::channel;
use futures::channel::oneshot::Canceled;
use futures::channel::oneshot::Receiver;
use futures::channel::oneshot::Sender;
use futures::future::join_all;
use futures::stream::FuturesUnordered;
use futures::Future;
use futures::SinkExt;

use crate::command::Command;
use crate::orchestrator::Message;
use crate::orchestrator::ServiceState;

// A definition that has not yet been run
#[derive(Clone, Debug)]
pub struct ServiceDef {
    pub name: String,
    pub same: Vec<String>,
    pub after: Vec<String>,
    pub start_command: Command,
    pub run_command: Command,
}

impl ServiceDef {
    // Starts services using an orchestrator and waits for them to start.
    pub async fn start_required_after(
        self,
        orchestrator: mpsc::Sender<Message>,
    ) -> Result<(), Canceled> {
        let orc = orchestrator.clone();
        let after = self
            .after
            .iter()
            .map(|service| {
                let mut orc = orc.clone();
                let (dep_sender, dep_receiver) = channel::<()>();
                async move {
                    orc.send(Message::DependOn {
                        service: service.clone(),
                        sender: dep_sender,
                    })
                    .await
                    .unwrap();
                    orc.send(Message::Start {
                        service: service.clone(),
                    })
                    .await
                    .unwrap();
                    dep_receiver.await
                }
            })
            .collect::<Vec<_>>();
        let receivers = join_all(after).await;
        receivers.into_iter().collect()
    }

    pub async fn start_wanted(self, orchestrator: mpsc::Sender<Message>) {
        let orc = orchestrator.clone();
        let same = self
            .same
            .iter()
            .map(|service| {
                let mut orc = orc.clone();
                async move {
                    orc.send(Message::Start {
                        service: service.clone(),
                    })
                    .await
                    .unwrap();
                }
            })
            .collect::<Vec<_>>();
        join_all(same).await;
    }
    pub async fn cancel_if_failed(self, orchestrator: mpsc::Sender<Message>) -> Vec<Receiver<()>> {
        let mut deps = self.same;
        deps.append(&mut self.after.clone());
        let futures = deps.iter().map(|service| {
            let (sender, receiver) = channel();
            async {
                println!("Sending failon message for {}", service.clone());
                orchestrator
                    .clone()
                    .send(Message::FailOn {
                        service: service.clone(),
                        sender,
                    })
                    .await
                    .unwrap();
                receiver
            }
        });
        join_all(futures).await
    }
    pub fn run(self, orchestrator: mpsc::Sender<Message>) -> impl Future<Output = ()> {
        println!("Run function");
        let mut orchestrator = orchestrator.clone();
        let run_cmd = self.run_command.clone();
        let start_cmd = self.start_command.clone();
        async move {
            println!("Running start cmd...");
            let result = start_cmd.run().await;
            println!("Finished starting..");
            match result {
                Ok(()) => {
                    orchestrator
                        .send(Message::Started {
                            service: self.name.clone(),
                            result: Ok(()),
                        })
                        .await
                        .unwrap();
                    let result = run_cmd.run().await;
                    orchestrator
                        .send(Message::Finished {
                            service: self.name.clone(),
                            result,
                        })
                        .await
                        .unwrap();
                }
                o => orchestrator
                    .send(Message::Started {
                        service: self.name.clone(),
                        result: o,
                    })
                    .await
                    .unwrap(),
            }
        }
    }
}
