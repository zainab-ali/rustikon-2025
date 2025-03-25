// Use channels to demonstrate actors
use futures::{join, SinkExt};
use std::collections::HashMap;
use std::sync::Arc;

use async_recursion::async_recursion;
use async_std::sync::Mutex;
use async_std::task;
use futures::channel::{mpsc, oneshot};
use futures::future::join_all;
use futures::Future;
use futures::{
    future::FusedFuture,
    pin_mut, select,
    stream::{FusedStream, Stream, StreamExt},
};

// Journey:
//  - Previously, we used an Arc<Mutex> to access shared state.
//  - We could instead not share state.
//  - Our futures each have their own state and they communicate via channels.
//  - We'll have one future "owning" the state, and other futures asking that future.
//  - It makes more sense to think of actors.
//  - We send messages between them via channels.
// We now move the state and locking into an orchestrator and communicate with it using channels.
// Rust is happy, and we don't get deadlocks, however we've realized that we aren't blocking when starting is in progress.
#[derive(Debug, Clone)]
struct UnitDef {
    name: String,
    requires_after: Vec<String>,
}

async fn start(name: String) {
    println!("Starting {}", name)
}

async fn run(name: String) {
    println!("Running {}", name)
}

// We now want to create an actor

enum Message {
    ShouldStart {
        name: String,
        sender: oneshot::Sender<bool>,
    },
    HasStarted {
        name: String,
    },
}

#[async_recursion]
async fn start_unit(units: Vec<UnitDef>, mut orchestrator: mpsc::Sender<Message>, name: String) {
    // We could get the state
    let (sender, receiver) = oneshot::channel::<bool>();
    orchestrator
        .send(Message::ShouldStart {
            name: name.clone(),
            sender,
        })
        .await
        .unwrap();

    if receiver.await.unwrap() {
        let unit = units
            .clone()
            .iter()
            .find(|unit| unit.name == name)
            .unwrap()
            .clone();
        let dependencies = unit
            .requires_after
            .iter()
            .map(|unit| start_unit(units.clone(), orchestrator.clone(), unit.clone()));
        join_all(dependencies).await;

        // Start ourselves
        start(unit.name.clone()).await;
        // And then insert
        orchestrator
            .send(Message::HasStarted { name: name.clone() })
            .await
            .unwrap();
    }
}

async fn orchestrator(mut inbox: mpsc::Receiver<Message>) {
    let mut state = HashMap::new();
    while let Some(message) = inbox.next().await {
        match message {
            Message::ShouldStart { name, sender } => match state.get(&name) {
                None => {
                    state.insert(name, ());
                    sender.send(true).unwrap();
                }
                Some(_) => sender.send(false).unwrap(),
            },
            Message::HasStarted { name } => (),
        }
    }
}

fn main() {
    let totoro_def = UnitDef {
        name: "Totoro".to_string(),
        requires_after: vec![],
    };
    let popcorn_def = UnitDef {
        name: "Popcorn".to_string(),
        requires_after: vec!["Totoro".to_string(), "Totoro".to_string()],
    };

    let mao_def = UnitDef {
        name: "Mao".to_string(),
        requires_after: vec!["Popcorn".to_string(), "Totoro".to_string()],
    };

    let units = vec![popcorn_def, mao_def, totoro_def];
    let (sender, receiver) = mpsc::channel(100);

    let orchestrator = orchestrator(receiver);
    let program = async { join!(orchestrator, start_unit(units, sender, "Mao".to_string())) };

    task::block_on(program);
}
