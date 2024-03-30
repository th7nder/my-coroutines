use std::sync::OnceLock;

use mio::{Events, Poll, Registry};

use crate::future::Future;

static REGISTRY: OnceLock<Registry> = OnceLock::new();
pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("called outside runtime context")
}

pub struct Runtime {
    poll: Poll
}

impl Runtime {
    pub fn new() -> Runtime {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        REGISTRY.set(registry).unwrap();
        Self {  
            poll
        }
    }

    pub fn block_on<F: Future>(&mut self, f: F) {
        let mut f = f;
        loop {
            match f.poll() {
                crate::future::PollState::NotReady => {
                    println!("Schedule other tasks");
                    let mut events = Events::with_capacity(1024);
                    self.poll.poll(&mut events, None).unwrap();
                },
                crate::future::PollState::Ready(_) => break,
            }
        }
    }
}