use async_std::sync::Mutex;
use futures::FutureExt;
use futures::{join, select};
use rand::prelude::*;
use std::{sync::Arc, time::Duration};

use futures_timer::Delay;

struct Spoon;
type SpoonMutex = Arc<Mutex<Spoon>>;
struct Pan;
type PanMutex = Arc<Mutex<Pan>>;

#[async_std::main]
async fn main() {
    breakfast().await;
}

async fn breakfast() {
    let spoon_mutex = find_spoon();
    let pan_mutex = find_pan();
    join!(
        cook_eggs(spoon_mutex.clone(), pan_mutex.clone()),
        fry_bacon(spoon_mutex, pan_mutex)
    );
}
fn find_spoon() -> SpoonMutex {
    Arc::new(Mutex::new(Spoon))
}
fn find_pan() -> PanMutex {
    Arc::new(Mutex::new(Pan))
}

async fn cook_eggs(spoon_mutex: SpoonMutex, pan_mutex: PanMutex) {
    let spoon = spoon_mutex.lock().await;
    crack_eggs().await;
    let pan = pan_mutex.lock().await;
    fry_eggs().await;
    drop(spoon);
    drop(pan);
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

async fn fry_bacon(spoon_mutex: SpoonMutex, pan_mutex: PanMutex) {
    let (spoon, pan) = join!(spoon_mutex.lock(), pan_mutex.lock());

    println!("Started frying bacon.");
    let mut timer = Box::pin(timer().fuse());
    let mut crisp_bacon = Box::pin(crisp_bacon().fuse());
    select! {
        () = timer => (),
        () = crisp_bacon => ()
    }
    println!("Finished frying bacon.");
    drop(spoon);
    drop(pan);
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
