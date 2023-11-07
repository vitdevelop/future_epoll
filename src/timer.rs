use std::future::Future;
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::FutureExt;
use log::debug;
use rustix::event::epoll::EventFlags;
use rustix::time::{timerfd_create, timerfd_gettime, timerfd_settime, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags};
use crate::context::CONTEXT;

pub(crate) async fn wait(seconds: i64)  {
    let timer = create_timer_fd(seconds).unwrap().boxed_local();
    timer.await.unwrap();

    debug!("Timer woke up")
}

fn create_timer_fd(seconds: i64) -> crate::Result<TimerFuture> {
    let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::NONBLOCK | TimerfdFlags::CLOEXEC)?;
    let mut time_spec = timerfd_gettime(&fd)?;
    time_spec.it_value.tv_sec = seconds;
    timerfd_settime(&fd, TimerfdTimerFlags::empty(), &time_spec)?;

    CONTEXT.with(|x| {
        let task_id = x.executor.current_task_id.clone().into_inner();
        x.epoll.add_task(&fd, task_id, EventFlags::IN | EventFlags::ONESHOT)
    })?;

    Ok(TimerFuture {
        fd,
    })
}

struct TimerFuture {
    fd: OwnedFd,
}

impl Future for TimerFuture {
    type Output = crate::Result<()>;

    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let ready = CONTEXT.with(|x| {
            let task_id = x.executor.current_task_id.clone().into_inner();
            let ready = x.epoll.is_ready(task_id);

            ready
        });

        if ready {
            CONTEXT.with(|x| {
                let task_id = x.executor.current_task_id.clone().into_inner();
                return x.epoll.remove_task(&self.fd, task_id);
            })?;
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }
}