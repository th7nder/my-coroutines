use std::{time::Instant};
mod future;
mod http;
mod runtime;
use future::{Future,PollState};
use http::Http;
use runtime::{Executor, Runtime, Waker};





fn main() {
    let now = Instant::now();
    let mut executor = runtime::init();
    let mut handles = vec![];

    for i in 1..8 {
        let executor = Executor::new();
        let name = format!("exec-{i}");
        let h = std::thread::Builder::new().name(name).spawn(move ||{
            executor.block_on(async_main());
        }).unwrap();
        handles.push(h);
    }
    executor.block_on(async_main());
    for handle in handles {
        handle.join().unwrap();
    }
    println!("ELAPSED: {} secs", now.elapsed().as_secs_f32());
}

    
coroutine fn request(i: usize) {
    let path = format!("/{}/HelloWorld", i * 1000);
    let txt = Http::get(&path).wait;
    let txt = txt.lines().last().unwrap().or_default();
    println!("{txt}");
}

coroutine fn async_main() {
    println!("Program starting");
    for i in 0..5 {
        runtime::spawn(request(i));
    }
}