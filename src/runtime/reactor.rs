use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    },
    task::{Context, Waker}
};

use mio::{net::TcpStream, Events, Interest, Poll, Registry, Token};


type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;
static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("called outside runtime context")
}

pub fn start() {
    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    let next_id = AtomicUsize::new(0);

    let reactor = Reactor {
        wakers: wakers.clone(),
        registry,
        next_id
    };
    REACTOR.set(reactor).ok().expect("Reactor has been running already");

    use std::thread::spawn;
    spawn(move || event_loop(poll, wakers));
}

pub struct Reactor {
    wakers: Wakers,
    next_id: AtomicUsize,
    registry: Registry,
}

impl Reactor {
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, id: usize) {
        self.registry.register(stream, Token(id), interest).unwrap();
    }

    pub fn set_waker(&self, id: usize, cx: &Context) {
        let _ = self
            .wakers
            .lock()
            .map(|mut w| w.insert(id, cx.waker().clone()))
            .unwrap();
    }

    pub fn deregister(&self, stream: &mut TcpStream, id: usize) {
        self.wakers.lock().map(|mut w| w.remove(&id)).unwrap();
        self.registry.deregister(stream).unwrap();
    }

    pub fn next_id(&self) -> usize {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }
}


fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(32);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            let Token(id) = event.token();
            let wakers = wakers.lock().unwrap();
            if let Some(waker) = wakers.get(&id) {
                waker.wake_by_ref();
            }
        }
    }
}