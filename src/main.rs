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

    




// =================================
// We rewrite this:
// =================================
    
// coroutine fn request(i: usize) {
//     let path = format!("/{}/HelloWorld", i * 1000);
//     let txt = Http::get(&path).wait;
//     let txt = txt.lines().last().unwrap().or_default();
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

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
        match self.state {
                State0::Start(i) => {
                    // ---- Code you actually wrote ----
                    let path = format!("/{}/HelloWorld", i * 1000);

                    // ---------------------------------
                    let fut1 = Box::new( Http::get(&path));
                    self.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.poll(waker) {
                        PollState::Ready(txt) => {
                            // ---- Code you actually wrote ----
                            let txt = txt.lines().last().unwrap_or_default();
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
//     println!("Program starting");
//     for i in 0..5 {
//         runtime::spawn(request(i));
//     }

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output=String> {
    Coroutine1::new()
}
        
enum State1 {
    Start,
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

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        loop {
        match self.state {
                State1::Start => {
                    // ---- Code you actually wrote ----
                    println!("Program starting");
    for i in 0..5 {
        runtime::spawn(request(i));
    }

                    // ---------------------------------
                    self.state = State1::Resolved;
                    break PollState::Ready(String::new());
                }

                State1::Resolved => panic!("Polled a resolved future")
            }
        }
    }
}
