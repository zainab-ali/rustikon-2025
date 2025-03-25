// We try using an ArcMutex to share state and end up with a deadlock.
use std::collections::HashMap;
use std::sync::Arc;

use async_recursion::async_recursion;
use async_std::sync::Mutex;
use async_std::task;
use futures::future::join_all;
use futures::Future;

// Journey:
//  - Previously, we could not borrow the HashMap as mutable.
//  - This is because Rust rightly will not let us borrow the mutable map in multiple places.
//  - To work around this, we use an Arc<Mutex>>
//  - An arc is an atomic reference count. Instead of telling the borrow checker to count references, we're counting them at runtime.
//  - This has some overhead, but needs must.
//  - A mutex lets us safely access and modify state across concurrent processes.
//  - We use it by locking. While we lock the mutex, no other future can update it.
//  - Is this correct?
//  - No! We have a deadlock.
//  - We've taken the lock, and so our dependencies can't make any updates.
//  - If you're used to concurrent thinking, you might have spotted this problem from a mile away.
//  - But most people aren't, or at least need to context switch (excuse the pun!).
// Learning objectives:
//  - We can use a Mutex to work with concurrent state, but we'll need to use automatic reference counting to share it.
//  - But we'll need to be careful to avoid deadlocks and other concurrency bugs.
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

// We create a mutable state to verify if Totoro is called twice.
#[async_recursion]
async fn start_unit(
    units: Vec<UnitDef>,
    mut_state: Arc<Mutex<HashMap<String, bool>>>,
    name: String,
) {
    // We could get the state
    println!("Waiting to aquire lock for {}", name);
    let mut state = mut_state.lock().await;
    println!("Acquired lock for {}", name);

    match state.get(&name.clone()) {
        None | Some(false) => {
            let unit = units
                .clone()
                .iter()
                .find(|unit| unit.name == name)
                .unwrap()
                .clone();
            let dependencies = unit
                .requires_after
                .iter()
                .map(|unit| start_unit(units.clone(), mut_state.clone(), unit.clone()));
            join_all(dependencies).await;

            // Start ourselves
            start(unit.name.clone()).await;
            // And then insert
            state.insert(name.clone(), true);
        }
        Some(true) => (),
    }
    println!("Released lock for {}", name);
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
    let state = Arc::new(Mutex::new(HashMap::new()));

    let program = async {
        start_unit(units, state, "Mao".to_string()).await;
    };

    task::block_on(program);
}
