use std::cell::RefCell;
use std::collections::HashMap;
use std::os::fd::{AsRawFd, OwnedFd};

use rustix::event::epoll::{create, CreateFlags, EventData, EventFlags, EventVec, wait};

use crate::Result;
use crate::waker::Task;

pub(crate) struct Epoll {
    fd: OwnedFd,
    // fd, task
    epoll_queue: RefCell<HashMap<u64, Task>>,
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
            let fd = event.data.u64();

            if let Some(task) = epoll_queue.remove(&fd) {
                task.as_waker().wake();
            }
        }

        Ok(())
    }

    pub(crate) fn add_task(&self, fd: &OwnedFd, task: Task, event_flags: EventFlags) -> Result<()> {
        let raw_fd = fd.as_raw_fd() as u64;
        rustix::event::epoll::add(&self.fd, fd, EventData::new_u64(raw_fd), event_flags)?;

        let mut map = self.epoll_queue.borrow_mut();

        if map.contains_key(&raw_fd) {
            return Err(Box::from(format!("Another task is waiting fd")));
        }
        map.insert(raw_fd, task);

        Ok(())
    }

    pub(crate) fn wait_task(&self, fd: &OwnedFd, task: Task) -> Result<()> {
        let mut map = self.epoll_queue.borrow_mut();
        let fd = fd.as_raw_fd() as u64;

        if map.contains_key(&fd) {
            return Err(Box::from(format!("Another task is waiting fd")));
        }
        map.insert(fd, task);

        Ok(())
    }

    pub(crate) fn modify_task(&self, fd: &OwnedFd, task: Task, event_flags: EventFlags) -> Result<()> {
        let raw_fd = fd.as_raw_fd() as u64;
        rustix::event::epoll::modify(&self.fd, fd, EventData::new_u64(raw_fd), event_flags)?;

        let mut map = self.epoll_queue.borrow_mut();

        if map.contains_key(&raw_fd) {
            return Err(Box::from(format!("Another task is waiting fd")));
        }
        map.insert(raw_fd, task);

        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) fn remove_task(&self, fd: &OwnedFd, task_id: u64) -> Result<()> {
        rustix::event::epoll::delete(&self.fd, fd)?;

        let mut map = self.epoll_queue.borrow_mut();
        map.remove(&task_id);

        Ok(())
    }

    pub(crate) fn add(&self, fd: &OwnedFd, event_flags: EventFlags) -> Result<()> {
        let raw_fd = fd.as_raw_fd() as u64;
        Ok(rustix::event::epoll::add(&self.fd, fd, EventData::new_u64(raw_fd), event_flags)?)
    }

    #[allow(dead_code)]
    pub(crate) fn modify(&self, fd: &OwnedFd, event_flags: EventFlags) -> Result<()> {
        let raw_fd = fd.as_raw_fd() as u64;
        Ok(rustix::event::epoll::modify(&self.fd, fd, EventData::new_u64(raw_fd), event_flags)?)
    }

    pub(crate) fn remove(&self, fd: &OwnedFd) -> Result<()> {
        rustix::event::epoll::delete(&self.fd, fd)?;

        Ok(())
    }
}