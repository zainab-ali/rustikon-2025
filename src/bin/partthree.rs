// Let's try and use a mutable hashmap to store state
// We run into issues with the borrow checker
use std::collections::HashMap;

use async_recursion::async_recursion;
use async_std::task;
use futures::future::join_all;
use futures::Future;

// Journey:
// - Our code still has a bug.
// - What if Mao and Popcorn both depend on Totoro?
// - Totoro would get triggered twice.
//
// - We need to keep track of some mutable state. Let's have a hashmap containing whether a unit has started or not.
// - If the unit has started, we enter true into the hashmap.
//
// - Each closure tries to borrow the mutable reference to state.
// - The borrow checker won't let us capture a mutable reference in more than one closures.
// - This is correct. What if we update both references at the same time? We would have a data race.
//
// Learning objectives:
// - We cannot share state across futures. The borrow checker correctly prevents us from doing so.
//
// We now wish to generalize the solution
// Any unit can start after any other unit.
// We have a UnitDef structure to represent units and their dependencies
// Assuming these are defined before we start systemd, how can we construct a task graph?
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
// 68 | ...|unit| start_unit(units.clone(), state, unit.clone()));
//    |         - ^^^^^^^^^^^^^^^^^^^^^^^^^^-----^^^^^^^^^^^^^^^
//    |         | |                         |
//    |         | |                         variable captured here
//    |         | returns a reference to a captured variable which escapes the closure body
//    |         inferred to be a `FnMut` closure
//    |
//    = note: `FnMut` closures only have access to their captured variables while they are executing...
//    = note: ...therefore, they cannot allow references to captured variables to escape
#[async_recursion]
async fn start_unit(units: Vec<UnitDef>, state: &mut HashMap<String, bool>, name: String) {
    // We could get the state
    match state.get(&name) {
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
                .map(|unit| start_unit(units.clone(), state, unit.clone()));
            // join_all(dependencies).await;

            // Start ourselves
            start(unit.name.clone()).await;
            // And then insert
            state.insert(name, true);
        }
        Some(true) => (),
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
    let mut state = HashMap::new();

    let program = async {
        start_unit(units, &mut state, "Mao".to_string()).await;
    };

    task::block_on(program);
}
