// Learning objectives:
// - Futures are structs, so must have a fixed size.
// - A recursive future is a bit like a recursive data structure: it needs to be boxed and put on the heap.
// - The async_recursion crate / macro deals with this for us.
// - There are lots of helpful crates in the async ecosystem that help us deal with this.

// Journey:
// There's a bug in our implementation. Can anyone spot it?
//  - We're not recursively starting units.
// What if we have this sequence?
// Mao
// Popcorn
// Totoro
// If we start Totoro, we're ignoring Mao. We need to call our function recursively.
// That looks easy enough to do, but if we try, our compiler blows up at us.
// If you're used to recursion with structs, you might have a gut feeling of what's going wrong here.
// A future is a struct, but what must that struct look like?
// It needs to refer to the futures that it's composed of. In the end, it needs to refer to itself.
// That means a few things. One is that its size is dynamic. Instead of being a specific impl Future (which has a size), it's an impl with a size unknown to the compiler.
// So we need to mark it as dynamic, and put it on the heap within a Box.
// In other words, this function must return a Box<dyn Future>>
// We can do this with boxed()
//
// There's an even easier way, which is using this async_recursion macro.
// As mentioned in the error message.

use std::time::Duration;

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
#[async_recursion]
async fn start_unit(units: Vec<UnitDef>, name: String) {
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
}

// Without boxing, we get the error:
// error[E0720]: cannot resolve opaque type
//   --> src/bin/parttwo.rs:80:57
//    |
// 43 |   async fn start(name: String, delay: u64) {
//    |                                            - returning this type `futures::future::Then<JoinAll<impl futures::Future<Output = ()>>, impl futures::Future<Output = ()>, [closure@src/bin/parttwo.rs:91:33: 91:41]>`
// ...
// 80 |   fn start_unit_fut(units: Vec<UnitDef>, name: String) -> impl Future<Output = (...
//    |                                                           ^^^^^^^^^^^^^^^^^^^^^^^^ recursive opaque type
fn start_unit_as_future(units: Vec<UnitDef>, name: String) -> BoxFuture<'static, ()> {
    let unit = units
        .clone()
        .iter()
        .find(|unit| unit.name == name)
        .unwrap()
        .clone();
    let dependencies = unit
        .requires_after
        .iter()
        .map(|unit| start_unit_as_future(units.clone(), unit.clone()));
    join_all(dependencies)
        .then(move |_| start(unit.name.clone(), unit.start_duration))
        .boxed()
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
