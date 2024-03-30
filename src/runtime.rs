use std::{cell::{Cell, RefCell}, collections::HashMap, sync::{Arc, Mutex}};

pub use executor::{spawn, Waker, Executor};
pub use reactor::{*};

use crate::future::Future;

pub mod executor;
pub mod reactor;


pub fn init() -> Executor {
    reactor::start();
    Executor::new()
}

pub struct Runtime {

}

impl Runtime {
    pub fn new() -> Runtime {
        Runtime {}
    }
    
    pub(crate) fn block_on(&self, fut: impl Future<Output = String>) {
        todo!()
    }
}