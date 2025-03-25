use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::FutureExt;
use futures::*;
use futures::{join, select};
use rand::prelude::*;
use std::time::Duration;

use futures_timer::Delay;

struct Spoon;
struct Pan;

enum Message {
    CrackEggs,
    FryEggs,
    FryBacon,
}

use Message::*;
#[async_std::main]
async fn main() {
    breakfast().await;
}

async fn breakfast() {
    let (sender, receiver) = channel::<Message>(10);
    join!(
        chef_actor(receiver),
        send_cook_eggs(sender.clone()),
        send_fry_bacon(sender),
    );
}

async fn chef_actor(mut receiver: Receiver<Message>) {
    let mut spoon = Spoon;
    let mut pan = Pan;
    while let Some(msg) = receiver.next().await {
        match msg {
            CrackEggs => crack_eggs(&mut spoon).await,
            FryEggs => fry_eggs(&mut spoon, &mut pan).await,
            FryBacon => fry_bacon(&mut spoon, &mut pan).await,
        }
    }
}

async fn send_cook_eggs(mut sender: Sender<Message>) {
    sender.send(CrackEggs).await.unwrap();
    sender.send(FryEggs).await.unwrap();
}

async fn send_fry_bacon(mut sender: Sender<Message>) {
    sender.send(FryBacon).await.unwrap();
}

async fn crack_eggs(_spoon: &mut Spoon) {
    println!("Started cracking egg.");
    random_sleep().await;
    println!("Finished cracking egg.");
}

async fn fry_eggs(_spoon: &mut Spoon, _pan: &mut Pan) {
    println!("Started frying egg.");
    random_sleep().await;
    println!("Finished frying egg.");
}
async fn fry_bacon(_spoon: &mut Spoon, _pan: &mut Pan) {
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
