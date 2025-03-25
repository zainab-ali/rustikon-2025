// Learning objectives:
// - async / await is actually just a notation for using futures.
// - Implementations of Future are just structures and can be manipulated as such.

// Journey:
// - Let's try and write a UnitDef that has a list of dependencies as values
// - We need to construct a list of futures.
// - We want to treat the futures as data.
// - Luckily we can, because futures are data.
// - A future is just a struct. You can think of the compiler constructing an anonymous struct for every async function: an anonymous implementation of future.
// - Let's take a look at our example of start_as_future: we can rewrite this with Future itself. It looks very messy, however.
// - So async / await is just this fancy syntax for then and map.
// - How does this relate to our code?
// - Looking at the futures as data, we want to create a collection of futures and await on them all together.
// - Thankfully, there's this function join_all that does that.
//
use std::time::Duration;

use async_std::task;
use futures::{future::join_all, Future, FutureExt};
use futures_timer::Delay;

#[derive(Clone)]
struct UnitDef {
    name: String,
    dependencies: Vec<String>,
}

async fn start(name: String) {
    println!("Starting {}", name);
    let () = Delay::new(Duration::from_secs(1)).await;
    println!("Started {}", name);
}

fn start_as_future(name: String, delay: u64) -> impl Future<Output = ()> {
    std::future::ready({
        println!("Starting {}", name);
    })
    .then(move |_| Delay::new(Duration::from_secs(delay)))
    .map(move |_| {
        println!("Started {}", name);
    })
}

// 27 | async fn start_unit(units: Vec<UnitDef>, name: String) {
// |                                                        ^ recursive `async fn`
// |
// = note: a recursive `async fn` must be rewritten to return a boxed `dyn Future`
// = note: consider using the `async_recursion` crate: https://crates.io/crates/async_recursion
fn find_unit(units: Vec<UnitDef>, name: String) -> UnitDef {
    units.iter().find(|unit| unit.name == name).unwrap().clone()
}
async fn start_unit(units: Vec<UnitDef>, name: String) {
    let unit = find_unit(units, name);
    let dependencies = unit
        .dependencies
        .iter()
        .map(|dep| start_unit(units.clone(), dep.clone()));
    join_all(dependencies).await;
    start(unit.name.clone()).await;
}

fn main() {
    let wifi = UnitDef {
        name: "wifi".to_string(),
        dependencies: vec![],
    };

    let firefox = UnitDef {
        name: "firefox".to_string(),
        dependencies: vec!["wifi".to_string()],
    };

    let webserver = UnitDef {
        name: "webserver".to_string(),
        dependencies: vec!["wifi".to_string()],
    };
    let slideshow = UnitDef {
        name: "slideshow".to_string(),
        dependencies: vec!["firefox".to_string(), "webserver".to_string()],
    };

    let units = vec![wifi, firefox, webserver, slideshow];

    let program = async {
        start_unit(units, "slideshow".to_string()).await;
    };

    task::block_on(program);

    println!("Part 2")
}
