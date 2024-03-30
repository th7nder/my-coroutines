use std::{time::Instant};
mod future;
mod http;
use future::*;
use crate::http::Http;

fn get_path(i: usize) -> String {
    format!("/{}/HelloWord{}", i * 1000, i)
}

coroutine fn request(i: usize) {
    let txt = Http::get(&get_path(i)).wait;
    println!("{txt}");
}

coroutine fn async_main() {
    println!("Program starting!");

    let mut vec = vec![];
    for i in 0..5 {
        vec.push(request(i));
    }
    join_all(vec).wait;
}

fn main() {
    let start = Instant::now();
    let mut fut = async_main();
    loop {
        match fut.poll() {
            PollState::Ready(_) => break,
            PollState::NotReady => ()
        }
    }
    println!("\nELAPSED TIME: {}", start.elapsed().as_secs_f32());
}
