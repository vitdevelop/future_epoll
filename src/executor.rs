use std::cell::{RefCell, UnsafeCell};
use std::future::Future;
use std::rc::Rc;
use std::task::Context;
use futures::FutureExt;
use futures::task::noop_waker_ref;
use crate::task::Task;

pub(crate) struct Executor {
    ready_queue: Rc<RefCell<Vec<Task>>>,
}

impl Executor {
    pub(crate) fn new() -> Executor {
        return Executor { ready_queue: Rc::new(RefCell::from(Vec::new())) };
    }

    pub(crate) fn spawn(&self, future: impl Future<Output=()> + 'static) {
        let future = future.boxed_local();
        let task = Task {
            future,
            sender: self.ready_queue.clone(),
        };
        self.ready_queue.borrow_mut().push(task)
    }

    pub(crate) fn run(&self) {
        self.execute_ready_tasks();
    }

    fn execute_ready_tasks(&self) {
        while let Some(task) = self.ready_queue.borrow_mut().pop() {
            let mut future = task.future;
            let waker = noop_waker_ref();
            let context = &mut Context::from_waker(waker);

            if future.as_mut().poll(context).is_pending() {
                self.ready_queue.borrow_mut().push(Task { future, sender: self.ready_queue.clone() })
            }
        }
    }
}