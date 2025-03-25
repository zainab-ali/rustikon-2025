// Learning objectives:
// - Encounter the async / await keywords
// - Encounter block_on
// - Encounter join
// - Think of async / await as a way of building a graph of work that happens in sequence and not.
// - Have a rough grasp of the difference between a task and a future.
use std::time::Duration;

use async_std::task;
use futures::join;
use futures_timer::Delay;
// We have two units: Mao and Popcorn.
// Mao has a Requires: Popcorn and After:Popcorn
// This means that the unit graph is like:
// Popcorn-start
// --------------------------
// Popcorn-run    Mao start
//                Mao run
//              \/

async fn start(name: String, delay: u64) {
    println!("Starting {}", name);
    // This Delay and duration are introduced so we can see the difference between running and starting.
    let () = Delay::new(Duration::from_secs(delay)).await;
    println!("Started {}", name);
}

async fn run(name: String) {
    println!("Running {}", name);
    let () = Delay::new(Duration::from_secs(1000)).await;
}

async fn popcorn_start() {
    start("Popcorn".to_string(), 1).await
}
async fn popcorn_run() {
    run("Popcorn".to_string()).await
}
async fn mao_start() {
    start("Mao".to_string(), 1).await
}
async fn mao_run() {
    run("Mao".to_string()).await
}

async fn program() {
    popcorn_start().await;
    let start_and_run_mao = async {
        mao_start().await;
        mao_run().await;
    };
    join!(popcorn_run(), start_and_run_mao);
}
fn main() {
    println!("Part 1");
    // Note: We need to explain the difference between tasks vs futures at this point.
    task::block_on(program());
}

/*
fn startup() {
    start_wifi();
    start_firefox();
    start_webserver();
    start_slideshow();
}
*/
