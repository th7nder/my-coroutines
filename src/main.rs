use std::{time::Instant};
mod future;
mod http;
use future::*;
use crate::http::Http;

fn get_path(i: usize) -> String {
    format!("/{}/HelloWord{}", i * 1000, i)
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


// =================================
// We rewrite this:
// =================================
    
// coroutine fn request(i: usize) {
//     let txt = Http::get(&get_path(i)).wait;
//     println!("{txt}");

// }

// =================================
// Into this:
// =================================

fn request(i: usize) -> impl Future<Output=String> {
    Coroutine0::new(i)
}
        
enum State0 {
    Start(usize),
    Wait1(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine0 {
    state: State0,
}

impl Coroutine0 {
    fn new(i: usize) -> Self {
        Self { state: State0::Start(i) }
    }
}


impl Future for Coroutine0 {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
        match self.state {
                State0::Start(i) => {
                    // ---- Code you actually wrote ----
                
                    // ---------------------------------
                    let fut1 = Box::new( Http::get(&get_path(i)));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll() {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----
                            println!("{txt}");

                            // ---------------------------------
                            self.state = State0::Resolved;
                            break PollState::Ready(String::new());
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State0::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}


// =================================
// We rewrite this:
// =================================
    
// coroutine fn async_main() {
//     println!("Program starting!");
// 
//     let mut vec = vec![];
//     for i in 0..5 {
//         vec.push(request(i));
//     }
//     join_all(vec).wait;

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output=String> {
    Coroutine1::new()
}
        
enum State1 {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Resolved,
}

struct Coroutine1 {
    state: State1,
}

impl Coroutine1 {
    fn new() -> Self {
        Self { state: State1::Start }
    }
}


impl Future for Coroutine1 {
    type Output = String;

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
        match self.state {
                State1::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting!");

    let mut vec = vec![];
    for i in 0..5 {
        vec.push(request(i));
    }

                    // ---------------------------------
                    let fut1 = Box::new(join_all(vec));
                    self.state = State1::Wait1(fut1);
                }

                State1::Wait1(ref mut f1) => {
                    match f1.poll() {
                        PollState::Ready(_) => {
                            // ---- Code you actually wrote ----
                        
                            // ---------------------------------
                            self.state = State1::Resolved;
                            break PollState::Ready(String::new());
                        }
                        PollState::NotReady => break PollState::NotReady,
                    }
                }

                State1::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}
