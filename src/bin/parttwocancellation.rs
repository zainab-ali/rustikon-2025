// Rust complains that we need to box the future.
// We can do this with the async_recursion package.
// But even if we do this, we end up with two Totoros running.

use std::{pin::Pin, time::Duration};

use async_recursion::async_recursion;
use async_std::task;
use futures::{
    future::{join_all, BoxFuture},
    Future, FutureExt,
};
use futures_timer::Delay;

#[derive(Clone)]
struct UnitDef {
    name: String,
    start_duration: u64,
    requires_after: Vec<String>,
}

async fn start(name: String, delay: u64) {
    println!("Starting {}", name);
    let () = Delay::new(Duration::from_secs(delay)).await;
    println!("Started {}", name);
}

fn start_unit(units: Vec<UnitDef>, name: String) -> Pin<Box<dyn Future<Output = ()>>> {
    Box::pin(async move {
        let unit = units
            .clone()
            .iter()
            .find(|unit| unit.name == name)
            .unwrap()
            .clone();
        let dependencies = unit
            .requires_after
            .iter()
            .map(|unit| start_unit(units.clone(), unit.clone()));
        join_all(dependencies).await;
        start(unit.name.clone(), unit.start_duration).await;
    })
}

fn main() {
    let totoro_def = UnitDef {
        name: "Totoro".to_string(),
        start_duration: 1,
        requires_after: vec![],
    };

    let popcorn_def = UnitDef {
        name: "Popcorn".to_string(),
        start_duration: 1,
        requires_after: vec!["Totoro".to_string()],
    };

    let mao_def = UnitDef {
        name: "Mao".to_string(),
        start_duration: 1,
        requires_after: vec!["Popcorn".to_string(), "Totoro".to_string()],
    };

    let units = vec![popcorn_def, mao_def, totoro_def];

    let program = async {
        start_unit(units, "Mao".to_string()).await;
    };

    task::block_on(program);

    println!("Part 2")
}
