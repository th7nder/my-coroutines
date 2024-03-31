use std::{marker::PhantomPinned, pin::Pin, time::Instant};
mod future;
mod http;
mod runtime;
use future::{Future,PollState};
use http::Http;
use runtime::Waker;
use std::fmt::Write;

fn main() {
    let now = Instant::now();
    let executor = runtime::init();
    executor.block_on(async_main());
    println!("ELAPSED: {} secs", now.elapsed().as_secs_f32());
}



// =================================
// We rewrite this:
// =================================
    
// coroutine fn async_main() {
//     println!("Program starting!");
//     let txt = Http::get("/600/HelloWorld1").wait;
//     println!("{txt}");
//     let txt = Http::get("/400/HelloWorld2").wait;
//     println!("{txt}");

// }

// =================================
// Into this:
// =================================

fn async_main() -> impl Future<Output=String> {
    Coroutine0::new()
}
        
enum State0 {
    Start,
    Wait1(Pin<Box<dyn Future<Output = String>>>),
    Wait2(Pin<Box<dyn Future<Output = String>>>),
    Resolved,
}

struct Coroutine0 {
    stack: Stack0,
    state: State0,
    _pin: PhantomPinned
}

#[derive(Default)]
struct Stack0 {
    // still don't get why we cant have self-referential structs
    buffer: Option<String>,
    writer: Option<*mut String>
}

impl Coroutine0 {
    fn new() -> Self {
        Self { state: State0::Start, stack: Stack0::default(), _pin: PhantomPinned }
    }
}


impl Future for Coroutine0 {
    type Output = String;

    fn poll(self: Pin<&mut Self>, waker: &Waker) -> PollState<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        loop {
        match this.state {
                State0::Start => {
                    // ---- Code you actually wrote ----
                    this.stack.buffer = Some(String::from("BUFFER:\n "));
                    this.stack.writer = Some(this.stack.buffer.as_mut().unwrap());
                    println!("Program starting!");

                    // ---------------------------------
                    let fut1 = Box::pin( Http::get("/600/HelloWorld1"));
                    this.state = State0::Wait1(fut1);
                }

                State0::Wait1(ref mut f1) => {
                    match f1.as_mut().poll(waker) {
                        PollState::Ready(txt) => {
                            let writer = unsafe { &mut *this.stack.writer.take().unwrap() };
                            // ---- Code you actually wrote ----
                            println!("{txt}");
                            writeln!(writer, "{txt}").unwrap();
                            // ---------------------------------
                            let fut2 = Box::pin( Http::get("/400/HelloWorld2"));
                            this.state = State0::Wait2(fut2);
                            this.stack.writer = Some(writer);
                        }
                        PollState::NotReady => break PollState::NotReady,
                 }
                }

                State0::Wait2(ref mut f2) => {
                    match f2.as_mut().poll(waker) {
                        PollState::Ready(txt) => {
                            // we need to take it as ref, as writer is pointing to this reference!! 
                            // if we move it out, the reference is invalid
                            let buffer = this.stack.buffer.as_ref().take().unwrap();
                            let writer = unsafe { &mut *this.stack.writer.take().unwrap() };
                            // ---- Code you actually wrote ----
                            println!("{txt}");
                            writeln!(writer, "{txt}").unwrap();

                            println!("Finished: {}", buffer);
                            // ---------------------------------
                            this.state = State0::Resolved;
                            // drop resources
                            let _ = this.stack.buffer.take();
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
