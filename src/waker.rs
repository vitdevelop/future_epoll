use std::task::{RawWaker, RawWakerVTable, Waker};
use log::debug;
use crate::context::CONTEXT;

#[derive(Clone, Hash, PartialEq, Eq)]
pub(crate) struct Task(u64);

impl Drop for Task {
    fn drop(&mut self) {
        debug!("Task {} dropped", self.0);
    }
}

impl Task {
    pub(crate) fn new(task_id: u64) -> Task {
        return Task(task_id);
    }

    fn as_raw_waker(&self) -> RawWaker {
        let ptr = task_to_ptr(self);
        return RawWaker::new(ptr, &WAKER_VTABLE);
    }

    pub(crate) fn as_waker(&self) -> Waker {
        unsafe {
            Waker::from_raw(self.as_raw_waker())
        }
    }

    fn notify(&self) {
        CONTEXT.with(|x| {
            x.executor.mark_task_ready(self.clone());
        });
    }
}

unsafe fn clone(ptr: *const ()) -> RawWaker {
    return ptr_to_task(ptr).clone().as_raw_waker();
}

unsafe fn wake(ptr: *const ()) {
    // waker has epoll queue lifetime
    wake_by_ref(ptr);
}

unsafe fn wake_by_ref(ptr: *const ()) {
    ptr_to_task(ptr).notify();
}

unsafe fn drop(_: *const ()) {}

const WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn ptr_to_task<'a>(ptr: *const ()) -> &'a Task {
    &*(ptr as *const Task)
}

fn task_to_ptr(current: &Task) -> *const () {
    current as *const Task as *const ()
}
