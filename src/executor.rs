use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use futures::FutureExt;
use futures::task::noop_waker_ref;
use log::warn;
use crate::context::CONTEXT;
use crate::Result;

pub(crate) type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output=T> + 'a>>;

pub(crate) trait Task {
    fn poll(self) -> Result<()>;
}

pub(crate) struct Executor<'a> {
    task_counter: u64,
    tasks: RefCell<HashMap<u64, LocalBoxFuture<'a, ()>>>,
    ready_queue: RefCell<VecDeque<u64>>,
    pub(crate) current_task_id: RefCell<u64>,
}

impl Executor<'_> {
    pub(crate) fn new() -> Executor<'static> {
        return Executor {
            task_counter: 0,
            tasks: RefCell::new(HashMap::default()),
            ready_queue: RefCell::new(VecDeque::new()),
            current_task_id: RefCell::new(0),
        };
    }

    fn get_task_id(&self) -> u64 {
        let mut task_id = self.task_counter;
        loop {
            if task_id > u64::MAX {
                task_id = 0;
            }
            task_id = task_id + 1;

            if !self.tasks.borrow().contains_key(&task_id) {
                return task_id;
            }
        }
    }

    pub(crate) fn spawn(future: impl Future<Output=()> + 'static) {
        let task = future.boxed_local();
        CONTEXT.with(|x| {
            let executor = &x.executor;
            let task_id = executor.get_task_id();
            if let Some(_) = executor.tasks.borrow_mut().insert(task_id, task) {
                warn!("A task is already was present when inserted new instead of");
            }

            executor.mark_task_ready(task_id);
        });
    }

    pub(crate) fn mark_task_ready(&self, task_id: u64) {
        self.ready_queue.borrow_mut().push_back(task_id);
    }

    pub(crate) fn run() -> Result<()> {
        while CONTEXT.with(|x| !x.executor.ready_queue.borrow().is_empty() ||
            !x.epoll.is_empty()) {
            CONTEXT.with(|x| x.executor.execute_ready_tasks())?;
            CONTEXT.with(|x| x.epoll.check_ready_tasks())?;
        }

        drop(CONTEXT);

        Ok(())
    }

    fn execute_ready_tasks(&self) -> Result<()> {
        while let Some(task_id) = self.ready_queue.borrow_mut().pop_front() {
            let mut tasks = self.tasks.borrow_mut();
            if let Some(task) = tasks.get_mut(&task_id) {
                self.current_task_id.replace(task_id);
                if task.as_mut().poll(&mut Context::from_waker(noop_waker_ref())).is_ready() {
                    tasks.remove(&task_id);
                }
            }
        }

        self.current_task_id.replace(0);

        Ok(())
    }
}