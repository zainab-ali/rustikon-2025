#![allow(dead_code)]

use futures::{
    future::{join_all, FutureExt}, // for `.fuse()`
    pin_mut,
    select,
    Future,
};
struct MyFut;

impl Future for MyFut {
    type Output = ();

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        todo!()
    }
}

async fn task() {
    println!("")
}

fn main() {
    // We fuse the tasks to let select know not to poll them again.
    // let t1 = task().fuse();
    // let t2 = task().fuse();
    // // Select takes a mutable reference to the future.
    // // It must be able to be unpin it, i.e. move it in memory.
    // // We surround the future in a "Pin", which keeps it somewhere.
    // let t1 = t1;
    // let mut t1 = Box::pin(t1);
    // let t2 = t2;
    // let x = join_all(vec![t1, t2]);
    // let mut t2 = Box::pin(t2);

    // let t3 = async {
    // 	select! {
    // 	() = t1 => println!("Task one"),
    // 	() = t2 => println!("Task 2"),
    // 	}
    // };
}
