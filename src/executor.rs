use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use futures::FutureExt;
use log::warn;
use crate::context::CONTEXT;
use crate::Result;
use crate::waker::Task;

pub(crate) type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output=T> + 'a>>;

pub(crate) struct Executor<'a> {
    task_counter: u64,
    tasks: RefCell<HashMap<Task, LocalBoxFuture<'a, Result<()>>>>,
    ready_queue: RefCell<VecDeque<Task>>,
    // futures can be embedded one into another, need to know who is running now to pass waker
    current_task: RefCell<Task>,
}

impl Executor<'_> {
    pub(crate) fn new() -> Executor<'static> {
        return Executor {
            task_counter: 0,
            tasks: RefCell::new(HashMap::default()),
            ready_queue: RefCell::new(VecDeque::new()),
            current_task: RefCell::new(Task::new(0)),
        };
    }

    pub(crate) fn get_current_task(&self) -> Task {
       return self.current_task.borrow().clone();
    }

    fn get_new_task(&self) -> Task {
        let mut task_id = self.task_counter;
        loop {
            if task_id > u64::MAX {
                task_id = 0;
            }
            task_id += 1;

            let task = Task::new(task_id);
            if !self.tasks.borrow().contains_key(&task) {
                return task;
            }
        }
    }

    pub(crate) fn spawn(future: impl Future<Output=Result<()>> + 'static) {
        let future = future.boxed_local();
        CONTEXT.with(|x| {
            let executor = &x.executor;
            let task = executor.get_new_task();
            if let Some(_) = executor.tasks.borrow_mut().insert(task.clone(), future) {
                warn!("A task is already was present when inserted new instead of");
            }

            executor.mark_task_ready(task);
        });
    }

    pub(crate) fn mark_task_ready(&self, task: Task) {
        self.ready_queue.borrow_mut().push_back(task);
    }

    pub(crate) fn run() -> Result<()> {
        while CONTEXT.with(|x| !x.executor.ready_queue.borrow().is_empty() ||
            !x.epoll.is_empty()) {
            CONTEXT.with(|x| x.executor.execute_ready_tasks())?;
            CONTEXT.with(|x| x.epoll.check_ready_tasks())?;
        }

        Ok(())
    }

    fn execute_ready_tasks(&self) -> Result<()> {
        while let Some(task) = self.ready_queue.borrow_mut().pop_front() {
            let mut tasks = self.tasks.borrow_mut();
            if let Some(future) = tasks.get_mut(&task) {
                self.current_task.replace(task.clone());
                if future.as_mut().poll(&mut Context::from_waker(&task.as_waker())).is_ready() {
                    tasks.remove(&task);
                }
            }
        }

        self.current_task.replace(Task::new(0));

        Ok(())
    }
}