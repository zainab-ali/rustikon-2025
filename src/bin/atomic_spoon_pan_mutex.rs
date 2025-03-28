use async_std::sync::Mutex;
use futures::FutureExt;
use futures::{join, select};
use rand::prelude::*;
use std::{sync::Arc, time::Duration};

use futures_timer::Delay;

struct Spoon;
struct Pan;
type SpoonAndPanMutex = Arc<Mutex<(Spoon, Pan)>>;

#[async_std::main]
async fn main() {
    breakfast().await;
}

async fn breakfast() {
    let mutex = find_spoon_and_pan();
    join!(
        cook_eggs(mutex.clone()),
        fry_bacon(mutex.clone())
    );
}
fn find_spoon_and_pan() -> SpoonAndPanMutex {
    Arc::new(Mutex::new((Spoon, Pan)))
}
async fn cook_eggs(mutex: SpoonAndPanMutex) {
    let spoon_and_pan = mutex.lock().await;
    crack_eggs().await;
    fry_eggs().await;
    drop(spoon_and_pan);
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

async fn fry_bacon(mutex: SpoonAndPanMutex) {
    let spoon_and_pan = mutex.lock().await;
    println!("Started frying bacon.");
    let mut timer = Box::pin(timer().fuse());
    let mut crisp_bacon = Box::pin(crisp_bacon().fuse());
    select! {
        () = timer => (),
        () = crisp_bacon => ()
    }
    println!("Finished frying bacon.");
    drop(spoon_and_pan);
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
