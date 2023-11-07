use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::OwnedFd;
use rustix::event::epoll::{create, CreateFlags, EventData, EventFlags, EventVec, wait};
use crate::context::CONTEXT;
use crate::Result;


pub(crate) struct Epoll {
    fd: OwnedFd,
    // fd, task_id
    epoll_queue: RefCell<HashMap<u64, bool>>,
}

impl Epoll {
    pub(crate) fn new() -> Result<Epoll> {
        let epoll_fd = create(CreateFlags::CLOEXEC)?;
        return Ok(Epoll {
            fd: epoll_fd,
            epoll_queue: RefCell::from(HashMap::new()),
        });
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.epoll_queue.borrow().is_empty()
    }

    pub(crate) fn check_ready_tasks(&self) -> Result<()> {
        let waiting_fds = self.epoll_queue.borrow().len();
        if waiting_fds == 0 {
            return Ok(());
        }
        let mut event_list = EventVec::with_capacity(waiting_fds);
        wait(&self.fd, &mut event_list, -1)?;

        let mut epoll_queue = self.epoll_queue.borrow_mut();
        for event in &event_list {
            let task_id = event.data.u64();

            epoll_queue.insert(task_id, true);
            CONTEXT.with(|x| {
                x.executor.mark_task_ready(task_id);
            })
        }

        Ok(())
    }

    pub(crate) fn is_ready(&self, task_id: u64) -> bool {
        let mut map = self.epoll_queue.borrow_mut();
        let ready = match map.get(&task_id) {
            None => { false }
            Some(ready) => { *ready }
        };

        if ready {
            map.remove(&task_id);
        }

        return ready;
    }

    pub(crate) fn add(&self, fd: &OwnedFd, task_id: u64, event_flags: EventFlags) -> Result<()> {
        rustix::event::epoll::add(&self.fd, fd, EventData::new_u64(task_id), event_flags)?;

        let mut map = self.epoll_queue.borrow_mut();
        map.insert(task_id, false);

        Ok(())
    }

    pub(crate) fn remove(&self, fd: &OwnedFd) -> Result<()> {
        rustix::event::epoll::delete(&self.fd, fd)?;

        Ok(())
    }
}