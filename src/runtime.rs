use std::{cell::{Cell, RefCell}, collections::HashMap, sync::{Arc, Mutex}};

pub use executor::{spawn, MyWaker, Executor};
pub use reactor::{*};


pub mod executor;
pub mod reactor;


pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}