use std::{thread::sleep, time::Duration};
use rand::prelude::*;

fn main() {
    breakfast();
}

fn breakfast() {
    cook_eggs();
    fry_bacon();
}
fn cook_eggs() {
    crack_eggs();
    fry_eggs();
}
fn crack_eggs() {
    println!("Started cracking egg.");
    random_sleep();
    println!("Finished cracking egg.");
    
}

fn fry_eggs() {
    println!("Started frying egg.");
    random_sleep();
    println!("Finished frying egg.");
    
}

fn fry_bacon() {
    println!("Started frying bacon.");
    random_sleep();
    println!("Finished frying bacon.");
    
}
fn random_sleep() {
    let mut rng = rand::rng();
    let time = rng.random_range(1..2);
    sleep(Duration::from_secs(time));
}
