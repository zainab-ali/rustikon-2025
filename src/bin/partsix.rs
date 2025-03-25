// Refactor to use a a manager actor and introduce select!
use futures::stream::FuturesUnordered;
use futures::{join, SinkExt};
use futures_timer::Delay;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use async_recursion::async_recursion;
use async_std::sync::Mutex;
use async_std::task;
use futures::channel::{mpsc, oneshot};
use futures::future::join_all;
use futures::Future;
use futures::FutureExt;
use futures::{
    future::FusedFuture,
    pin_mut, select,
    stream::{FusedStream, Stream, StreamExt},
};

// We block while starting is in progress by using a channel for communication.
//
// Notes for this section:
// We now wish to generalize the solution
// Any unit can start after any other unit.
// We have a UnitDef structure to represent units and their dependencies
// Assuming these are defined before we start systemd, how can we construct a task graph?
//
// We end up wanting a recursive future start_unit, and hit into a wall with rust.
// error[E0733]: recursion in an `async fn` requires boxing
//   --> src/bin/parttwo.rs:20:56
//    |
// 20 | async fn start_unit(units: Vec<UnitDef>, name: String) {
//    |                                                        ^ recursive `async fn`
//    |
//    = note: a recursive `async fn` must be rewritten to return a boxed `dyn Future`
//    = note: consider using the `async_recursion` crate: https://crates.io/crates/async_recursion
//
// Even if we solve this problem, we will hit a wall in our starting and running:
//
// Assuming Mao has a dependency on Popcorn and Totoro, Popcorn should be able to run at the same time as Totoro starting.
//
// Popcorn start | Totoro start |          |
// ------------- |              |          |
// Popcorn run   |------------- |----------|
//               | Totoro run   | Mao start
//
// This can be resolved by complex select logic, if we so choose.
//
// Even if we resolve this, there's another problem:
//  What about shared dependencies? What if Mao and Popcorn both depend on Totoro?
// Totoro would get triggered twice.
//
//
#[derive(Debug, Clone)]
struct UnitDef {
    name: String,
    start_duration: u64,
    run_duration: u64,
    requires_after: Vec<String>,
}

async fn start(name: String, delay: u64) {
    println!("Starting {}", name);
    // This Delay and duration are introduced so we can see the difference between running and starting.
    let () = Delay::new(Duration::from_secs(delay)).await;
    println!("Started {}", name);
}

async fn run(name: String, delay: u64) {
    println!("Running {}", name);
    // This Delay and duration are introduced so we can see the difference between running and starting.
    let () = Delay::new(Duration::from_secs(delay)).await;
    println!("Ran {}", name);
}

// We now want to create an actor

enum Message {
    Start {
        name: String,
        sender: oneshot::Sender<()>,
        canceller: oneshot::Sender<()>,
    },
    HasStarted {
        name: String,
    },
}

#[async_recursion]
async fn start_unit(units: Vec<UnitDef>, mut orchestrator: mpsc::Sender<Message>, name: String) {
    let unit = units
        .clone()
        .iter()
        .find(|unit| unit.name == name)
        .unwrap()
        .clone();
    // Tell the manager to start a dependency.
    let dependencies = unit.requires_after.iter().map(|unit| async {
        let (sender, receiver) = oneshot::channel();
        let (cancel_sender, cancel_receiver) = oneshot::channel();
        orchestrator
            .clone()
            .send(Message::Start {
                name: unit.clone(),
                sender,
                canceller: cancel_sender,
            })
            .await
            .unwrap();
        receiver.await.unwrap();
        cancel_receiver
    });
    let cancellations = join_all(dependencies).await;

    // Start ourselves
    let program = async {
        start(name.clone(), unit.start_duration).await;
        // And then insert
        orchestrator
            .send(Message::HasStarted { name: name.clone() })
            .await
            .unwrap();
        // TODO: Add cancellation on error.
        run(unit.name.clone(), unit.run_duration).await;
    };

    let mut cancellations = FuturesUnordered::from_iter(cancellations);
    let program = program.fuse();
    pin_mut!(program);

    loop {
        select! {
            () = program => break,
        _ = cancellations.select_next_some() => break,
        }
    }
}

enum UnitState {
    Starting { waiters: Vec<oneshot::Sender<()>> },
    Started,
}
// How is this an actor? The only thing we're waiting on is a message.
async fn orchestrator(
    mut inbox: mpsc::Receiver<Message>,
    mut manager: mpsc::Sender<ManagerMessage>,
) {
    let mut state = HashMap::new();
    // Note that we can represent the state of each unit as a state machine.
    // What if we want to start a unit after it has finished?
    while let Some(message) = inbox.next().await {
        match message {
            Message::Start {
                name,
                sender,
                canceller,
            } => match state.get_mut(&name) {
                None => {
                    // Trigger a task to be started
                    manager
                        .send(ManagerMessage::Start { name: name.clone() })
                        .await
                        .unwrap();
                    let v: Vec<oneshot::Sender<()>> = vec![sender];
                    state.insert(name, UnitState::Starting { waiters: v });
                }
                Some(UnitState::Starting { waiters }) => {
                    waiters.push(sender);
                }
                Some(UnitState::Started) => sender.send(()).unwrap(),
            },
            Message::HasStarted { name } => {
                let unit_state = state.get_mut(&name).unwrap();
                match unit_state {
                    UnitState::Starting { waiters } => {
                        while let Some(waiter) = waiters.pop() {
                            waiter.send(()).unwrap();
                        }
                        *unit_state = UnitState::Started;
                    }
                    UnitState::Started => (),
                }
            }
        }
    }
}

enum ManagerMessage {
    Start { name: String },
}

async fn manager(
    mut inbox: mpsc::Receiver<ManagerMessage>,
    units: Vec<UnitDef>,
    orchestrator: mpsc::Sender<Message>,
) {
    let mut services = FuturesUnordered::new();
    loop {
        select! {
        maybe_message = inbox.next() => {
        match maybe_message {
            Some(ManagerMessage::Start { name }) => {
        println!("Received start message for {:?}", name.clone());
        let service = start_unit(units.clone(), orchestrator.clone(), name.clone());
        println!("Started service {:?}", name);
            services.push(service);
            },
            None => ()
        }
        },
            () = services.select_next_some() => {
        println!("A service finished.");
            }
        }
    }
}

fn main() {
    let totoro_def = UnitDef {
        name: "Totoro".to_string(),
        start_duration: 1,
        run_duration: 2,
        requires_after: vec![],
    };
    let popcorn_def = UnitDef {
        name: "Popcorn".to_string(),
        start_duration: 1,
        run_duration: 2,
        requires_after: vec!["Totoro".to_string(), "Totoro".to_string()],
    };

    let mao_def = UnitDef {
        name: "Mao".to_string(),
        start_duration: 1,
        run_duration: 2,
        requires_after: vec!["Popcorn".to_string(), "Totoro".to_string()],
    };

    let units = vec![popcorn_def, mao_def, totoro_def];
    let (mut sender, receiver) = mpsc::channel(100);
    let (manager_sender, manager_receiver) = mpsc::channel(100);

    let orchestrator = orchestrator(receiver, manager_sender);
    let manager = manager(manager_receiver, units, sender.clone());
    let program = async {
        let (start_sender, receiver) = oneshot::channel();
        let (cancel_sender, cancel_receiver) = oneshot::channel();
        let res = sender.send(Message::Start {
            name: "Mao".to_string(),
            sender: start_sender,
            canceller: cancel_sender,
        });
        join!(manager, orchestrator, res, receiver)
    };

    task::block_on(program);
}
