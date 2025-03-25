#![allow(dead_code)]

mod command;
mod daemon;
mod newunit;
mod orchestrator;
mod unit;
use futures::join;
use futures::select;
use std::io::prelude::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;
use std::time::Duration;
mod deferred;
use async_recursion::async_recursion;

use deferred::Deferred;
use futures::channel::mpsc;
use futures::future::select;
use futures::future::BoxFuture;
use futures::FutureExt;
use futures::SinkExt;
use futures::StreamExt;
use std::collections::hash_map::{Entry, HashMap};
use std::io::Error;
use std::io::ErrorKind;

use async_std::{
    net::{TcpListener, ToSocketAddrs},
    prelude::*,
    task,
};

use crate::command::Command;
use crate::newunit::ServiceDef;
use crate::orchestrator::Message;
use crate::orchestrator::Orchestrator;
use crate::orchestrator::ServiceState;

async fn cook_bacon() {
    println!("Cooking bacon...");
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Cooked bacon");
}

async fn crack_egg() {
    println!("Cracking egg...");
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Cracked egg");
}

async fn fry_egg() {
    println!("Frying egg...");
    crack_egg().await;
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Fried egg");
}

async fn example_await() {
    fry_egg().await;
    cook_bacon().await;
}

async fn example_join() {
    join!(fry_egg(), cook_bacon());
}

async fn example_select() {
    let timer = async {
        println!("Starting bacon timer...");
        async_std::task::sleep(Duration::from_secs(1)).await;
        println!("Finished bacon timer.");
    };
    let mut timer_pinned = Box::pin(timer.fuse());
    let mut cook_bacon_pinned = Box::pin(cook_bacon().fuse());

    select! {
        _ = timer_pinned => (),
        _ = cook_bacon_pinned => ()
    };
}

struct Spoon;

async fn crack_egg_with_spoon(spoon: Arc<Mutex<Spoon>>) {
    println!("Cracking egg...");
    let guard = spoon.lock().unwrap();
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Cracked egg...");
}

// Example of a reentrant deadlock
async fn fry_egg_with_spoon(spoon: Arc<Mutex<Spoon>>) {
    println!("Cracking egg...");
    let guard = spoon.lock().unwrap();
    crack_egg_with_spoon(spoon.clone()).await;
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Cracked egg...");
}

// Example of a how to correct a reentrant deadlock
async fn fry_egg_with_spoon_corrected(spoon: Arc<Mutex<Spoon>>) {
    println!("Cracking egg...");
    crack_egg_with_spoon(spoon.clone()).await;
    let guard = spoon.lock().unwrap();
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Cracked egg...");
}

struct Pan;
async fn fry_egg_with_spoon_and_pan(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
    println!("Frying egg...");
    crack_egg_with_spoon(spoon.clone()).await;
    let (spoon_guard, pan_guard) = join!(async { spoon.lock().unwrap() }, async {
        pan.lock().unwrap()
    },);
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Fried egg...");
}

async fn fry_bacon_with_spoon_and_pan(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
    println!("Frying bacon...");
    let (spoon_guard, pan_guard) = join!(async { spoon.lock().unwrap() }, async {
        pan.lock().unwrap()
    },);
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Fried bacon...");
}

// Example of a deadlock
async fn fry_egg_and_bacon_with_spoon_and_pan(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
    let fry_egg = fry_egg_with_spoon_and_pan(spoon.clone(), pan.clone());
    let fry_bacon = fry_bacon_with_spoon_and_pan(spoon.clone(), pan.clone());
    join!(fry_egg, fry_bacon);
}

async fn fry_bacon_with_spoon_and_pan_retry(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
        let (spoon_guard, pan_guard) = loop {
        match (spoon.try_lock(), pan.try_lock()) {
            (Ok(spoon_guard), Ok(pan_guard)) => {
                break (spoon_guard, pan_guard);
            }
            (Ok(spoon_guard), Err(_)) => {
                drop(spoon_guard);
            }
            (Err(_), Ok(pan_guard)) => {
                drop(pan_guard);
            }
            (Err(_), Err(_)) => (),
        };
    };
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Fried bacon...");
}

async fn fry_egg_with_spoon_and_pan_retry(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
    let (spoon_guard, pan_guard) = loop {
        match (spoon.try_lock(), pan.try_lock()) {
            (Ok(spoon_guard), Ok(pan_guard)) => {
                break (spoon_guard, pan_guard);
            }
            (Ok(spoon_guard), Err(_)) => {
                drop(spoon_guard);
            }
            (Err(_), Ok(pan_guard)) => {
                drop(pan_guard);
            }
            (Err(_), Err(_)) => (),
        };
    };
    async_std::task::sleep(Duration::from_secs(1)).await;
    println!("Fried egg...");
}

// Example of a livelock
async fn fry_egg_and_bacon_with_spoon_and_pan_retry(spoon: Arc<Mutex<Spoon>>, pan: Arc<Mutex<Pan>>) {
    let fry_bacon = fry_bacon_with_spoon_and_pan_retry(spoon.clone(), pan.clone());
    let fry_egg = fry_egg_with_spoon_and_pan_retry(spoon.clone(), pan.clone());
    join!(fry_bacon, fry_egg);
}

// TODO: Example of an actor



fn main() -> std::io::Result<()> {
    // Next steps:
    // - Try out error handling: failure and also dropping on error / require pairing.
    Ok(())
}
