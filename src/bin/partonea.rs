use std::thread::sleep;
use std::time::Duration;

fn start<'a>(name: &'a str) {
    println!("Starting {}", name);
    sleep(from_secs(1)); // Never do this
    println!("Started {}", name);
}

fn startup() {
    start("wifi");
    start("firefox");
    start("webserver");
    start("slideshow");
}

fn main() {
    startup();
}
