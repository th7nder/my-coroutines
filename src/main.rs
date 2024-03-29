use std::{thread, time::Duration};

use future::{Future, PollState};

use crate::http::Http;

mod future;
mod http;

struct Coroutine {
    state: State,
}

enum State {
    Start,
    Wait1(Box<dyn Future<Output = String>>),
    Wait2(Box<dyn Future<Output = String>>),
    Resolved,
}

impl Coroutine {
    fn new() -> Coroutine {
        Coroutine {
            state: State::Start,
        }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> future::PollState<Self::Output> {
        loop {
            match self.state {
                State::Start => {
                    println!("Program starting");
                    self.state = State::Wait1(Box::new(Http::get("/1000/First")));
                }
                State::Wait1(ref mut fut) => match fut.poll() {
                    PollState::Ready(output) => {
                        println!("Received: {}", output);

                        self.state = State::Wait2(Box::new(Http::get("/2000/Next")));
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Wait2(ref mut fut) => match fut.poll() {
                    PollState::Ready(output) => {
                        println!("Received: {}", output);

                        self.state = State::Resolved;
                        break future::PollState::Ready(())
                    }
                    PollState::NotReady => break PollState::NotReady,
                },
                State::Resolved => panic!("polled a resolved future")
            }
        }
    }
}

fn async_main() -> impl Future<Output = ()> {
    Coroutine::new()
}

fn main() {
    let mut fut = async_main();
    loop {
        match fut.poll() {
            PollState::Ready(()) => break,
            PollState::NotReady => {
                println!("Schedule other work");
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
