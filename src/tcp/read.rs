use std::future::Future;
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use rustix::event::epoll::EventFlags;
use rustix::io;
use rustix::net::{recv, RecvFlags};
use crate::context::CONTEXT;
use crate::Result;
use crate::tcp::TcpClient;

pub(crate) struct TcpReadFuture<'a, 'b> {
    fd: &'b OwnedFd,
    buffer: &'a mut [u8],
}

impl TcpReadFuture<'_, '_> {
    pub(super) fn new<'a, 'b>(fd: &'b OwnedFd, buffer: &'a mut [u8]) -> Result<TcpReadFuture<'a, 'b>> {
        Ok(TcpReadFuture {
            fd,
            buffer,
        })
    }
}

impl Future for TcpReadFuture<'_, '_> {
    type Output = Result<usize>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fd = self.fd;
        return match recv(&fd, self.buffer, RecvFlags::empty()) {
            Ok(size) => {
                Poll::Ready(Ok(size))
            }
            Err(e) => {
                if e == io::Errno::AGAIN || e == io::Errno::WOULDBLOCK {
                    CONTEXT.with(|x| {
                        let task_id = x.executor.current_task_id.clone().into_inner();
                        x.epoll.modify_task(self.fd, task_id, EventFlags::IN | EventFlags::ONESHOT)
                    })?;

                    Poll::Pending
                } else {
                    return Err(e)?;
                }
            }
        };
    }
}

impl TcpClient {
    pub(crate) async fn read(&self, buffer: &mut [u8]) -> Result<usize> {
        TcpReadFuture::new(&self.fd, buffer)?.await
    }
}