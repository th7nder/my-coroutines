use std::{marker::PhantomPinned, pin::Pin, time::Instant};
mod http;
mod runtime;
use http::Http;

fn main() {
    let now = Instant::now();
    let executor = runtime::init();
    executor.block_on(async_main());
    println!("ELAPSED: {} secs", now.elapsed().as_secs_f32());
}

async fn async_main() {
    let txt = Http::get("/600/HelloWorld").await;
    println!("{txt}");

    let txt = Http::get("/400/HelloWorld").await;
    println!("{txt}");
}