use futures::join;
use futures_timer::Delay;
use std::time::Duration;

async fn start<'a>(name: &'a str) {
    println!("Starting {}", name);
    let () = Delay::new(Duration::from_secs(1)).await;
    println!("Started {}", name);
}

#[async_std::main]
async fn main() {
    start("wifi").await;
    join!(start("firefox"), start("webserver"));
    start("slideshow").await;
}
