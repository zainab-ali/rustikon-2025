use futures::join;
use rand::prelude::*;
use std::time::Duration;

use futures_timer::Delay;

#[async_std::main]
async fn main() {
    breakfast().await;
}

async fn breakfast() {
    join!(cook_eggs(), fry_bacon());
}
async fn cook_eggs() {
    crack_eggs().await;
    fry_eggs().await;
}
async fn crack_eggs() {
    println!("Started cracking egg.");
    random_sleep().await;
    println!("Finished cracking egg.");
}

async fn fry_eggs() {
    println!("Started frying egg.");
    random_sleep().await;
    println!("Finished frying egg.");
}

async fn fry_bacon() {
    println!("Started frying bacon.");
    random_sleep().await;
    println!("Finished frying bacon.");
}
async fn random_sleep() {
    let mut rng = rand::rng();
    let time = rng.random_range(1..2);
    Delay::new(Duration::from_secs(time)).await;
}
