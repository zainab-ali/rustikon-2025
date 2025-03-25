use futures::{join, select};
use rand::prelude::*;
use std::{time::Duration};
use futures::FutureExt;

use futures_timer::Delay;

struct Spoon;

#[async_std::main]
async fn main() {
    breakfast().await;
}

async fn breakfast() {
    let mut spoon = Spoon;
    join!(cook_eggs(&mut spoon), fry_bacon(&mut spoon));
}
async fn cook_eggs(spoon: &mut Spoon) {
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

async fn fry_bacon(spoon: &mut Spoon) {
    println!("Started frying bacon.");
    let mut timer = Box::pin(timer().fuse());
    let mut crisp_bacon = Box::pin(crisp_bacon().fuse());
    select! {
        () = timer => (),
        () = crisp_bacon => ()
    }
    println!("Finished frying bacon.");
}
async fn timer() {
    println!("Started timer.");
    Delay::new(Duration::from_secs(1)).await;
    println!("Finished timer.");
}

async fn crisp_bacon() {
    println!("Started crisping bacon.");
    random_sleep().await;
    println!("Finished crisping bacon.");
}

async fn random_sleep() {
    let mut rng = rand::rng();
    let time = rng.random_range(1..2);
    Delay::new(Duration::from_secs(time)).await;
}
