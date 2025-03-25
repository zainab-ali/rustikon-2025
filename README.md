# Reasoning with Async Rust, Rustikon 2025

This repo contains the source code for [Reasoning with Async Rust, presented at Rustikon 2025](https://kebab-ca.se/presentations/2025-03-26-reasoning-with-async-rust.html).

Look through the binaries in the following order:
 - `synchronous`: code with vanilla synchronous Rust.
 - `async_await`: introduces the `async` and `await` keywords.
 - `join`: introduces the `join!` macro.
 - `select`: introduces the `select!` macro.
 - `mutable`: an example of how mutable borrows cannot be used.
 - `shared_spoon_mutex`: introduces a mutex.
 - `shared_spoon_pan_mutex`: demonstrates a deadlock.
 - `atomic_spoon_pan_mutex`: a solution to the deadlock.
 - `actor`: an example of an actor-like approach.
