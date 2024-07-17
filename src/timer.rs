use std::future::Future;
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::FutureExt;
use log::debug;
use rustix::event::epoll::EventFlags;
use rustix::time::{timerfd_create, timerfd_gettime, timerfd_settime, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags};
use crate::context::CONTEXT;

pub(crate) async fn wait(seconds: i64) {
    let timer = create_timer_fd(seconds).unwrap().boxed_local();
    timer.await.unwrap();

    debug!("Timer woke up")
}

fn create_timer_fd(seconds: i64) -> crate::Result<TimerFuture> {
    let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::NONBLOCK | TimerfdFlags::CLOEXEC)?;

    Ok(TimerFuture {
        fd,
        seconds,
        ready: false,
    })
}

struct TimerFuture {
    fd: OwnedFd,
    seconds: i64,
    ready: bool,
}

impl Future for TimerFuture {
    type Output = crate::Result<()>;

    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.ready {
            let mut time_spec = timerfd_gettime(&self.fd)?;
            time_spec.it_value.tv_sec = self.seconds;
            timerfd_settime(&self.fd, TimerfdTimerFlags::empty(), &time_spec)?;

            CONTEXT.with(|x| {
                let task = x.executor.get_current_task();
                x.epoll.add_task(&self.fd, task, EventFlags::IN | EventFlags::ONESHOT)
            })?;

            // should fire when epoll received fd
            self.ready = true;
            return Poll::Pending;
        } else {
            CONTEXT.with(|x| {
                return x.epoll.remove(&self.fd);
            })?;
            Poll::Ready(Ok(()))
        }
    }
}