#![allow(dead_code)]

use std::cell::Ref;

use async_std::channel::Sender;

fn is_sync<T: Sync>(t: T) {}

struct Person<'a> {
    name: &'a str,
}

fn main() {
    is_sync(Person { name: "Mao" });
    is_sync(42);
    let v = "Mao";
    is_sync(v);
    let f = async { 1 };
    is_sync(f);

    let mut s = "Mao".to_string();
    let z = s.push_str("foo");
    let raw: *const String = std::ptr::null();

    // We can create a futrue that is not thread safe, for running on a single-threaded executor.
    let foo = async {
        let x = raw;
    };

    let x: Sender<i32> = unimplemented!();
    // x.send(1)

    // is_sync(foo)
    // unsafe {
    // 	// Segfaults
    // 	//println!("{:?}", *raw);
    // 	let y = *raw;
    // 	let z = &y;
    // }
    // }
    // is_sync(raw);
}
