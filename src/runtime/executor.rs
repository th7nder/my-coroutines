use crate::future::{Future, PollState};
use std::{
    cell::{Cell, RefCell}, collections::HashMap, pin::Pin, sync::{Arc, Mutex}, thread::{self, Thread}
};

pub struct Executor;

impl Executor {
    pub fn new() -> Executor {
        Executor
    }

    pub fn pop_ready(&self) -> Option<usize> {
        CURRENT_EXEC.with(|e: &ExecutorCore| e.ready_queue.lock().map(|mut q| q.pop()).unwrap())
    }

    pub fn get_future(&self, id: usize) -> Option<Task> {
        CURRENT_EXEC.with(|e: &ExecutorCore| e.tasks.borrow_mut().remove(&id))
    }

    pub fn get_waker(&self, id: usize) -> Waker {
        Waker {
            thread: thread::current(),
            id,
            ready_queue: CURRENT_EXEC.with(|e: &ExecutorCore| e.ready_queue.clone()),
        }
    }

    pub fn insert_task(&self, id: usize, task: Task) {
        CURRENT_EXEC.with(|e: &ExecutorCore| e.tasks.borrow_mut().insert(id, task));
    }

    pub fn task_count(&self) -> usize {
        CURRENT_EXEC.with(|e: &ExecutorCore| e.tasks.borrow().len())
    }

    pub fn block_on<F>(&self, future: F)
    where
        F: Future<Output = String> + 'static,
    {
        spawn(future);
        loop {
            while let Some(ready_id) = self.pop_ready() {
                let mut future = match self.get_future(ready_id) {
                    Some(f) => f,
                    None => continue,
                };

                let waker = self.get_waker(ready_id);
                match future.as_mut().poll(&waker) {
                    PollState::Ready(_) => continue,
                    PollState::NotReady => {
                        self.insert_task(ready_id, future);
                        continue;
                    }
                }
            }
            let task_count = self.task_count();
            let thread_name = thread::current().name().unwrap_or_default().to_string();
            if task_count > 0 {
                println!("{thread_name}: waiting for {task_count} tasks");
                thread::park();
            } else {
                println!("{thread_name}: finished, exiting...");
                break;
            }
        }
    }
}

#[derive(Clone)]
pub struct Waker {
    thread: Thread,
    id: usize,
    ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut q| q.push(self.id))
            .unwrap();

        self.thread.unpark();
    }
}

type Task = Pin<Box<dyn Future<Output = String>>>;
thread_local! {
    static CURRENT_EXEC: ExecutorCore = ExecutorCore::default();
}

#[derive(Default)]
struct ExecutorCore {
    // can't mutate a static variable, so that's why we need a RefCell
    tasks: RefCell<HashMap<usize, Task>>,
    // waker will be sent to a different thread? woot?
    ready_queue: Arc<Mutex<Vec<usize>>>,
    // can't mutate a static thing, that's why we need a Cell (replace, not mutate)
    next_id: Cell<usize>,
}

pub fn spawn<F>(future: F)
// 'static lifetime means it may be able to live through entire program
where
    F: Future<Output = String> + 'static,
{
    CURRENT_EXEC.with(|e: &ExecutorCore| {
        let id = e.next_id.get();
        e.next_id.set(id + 1);

        e.tasks.borrow_mut().insert(id, Box::pin(future));
        // it needs to be polled at least once
        e.ready_queue.lock().map(|mut q| q.push(id)).unwrap();
    })
}
