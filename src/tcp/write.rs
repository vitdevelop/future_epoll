use std::future::Future;
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use rustix::event::epoll::EventFlags;
use rustix::io;
use rustix::io::write;
use crate::context::CONTEXT;
use crate::Result;
use crate::tcp::TcpClient;

pub(crate) struct TcpWriteFuture<'a, 'b> {
    fd: &'b OwnedFd,
    buffer: &'a [u8],
}

impl TcpWriteFuture<'_, '_> {
    pub(super) fn new<'a, 'b>(fd: &'b OwnedFd, buffer: &'a [u8]) -> Result<TcpWriteFuture<'a, 'b>> {
        Ok(TcpWriteFuture {
            fd,
            buffer,
        })
    }
}

impl Future for TcpWriteFuture<'_, '_> {
    type Output = Result<usize>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        let fd = self.fd;
        return match write(&fd, self.buffer) {
            Ok(size) => {
                Poll::Ready(Ok(size))
            }
            Err(e) => {
                if e == io::Errno::AGAIN || e == io::Errno::WOULDBLOCK {
                    CONTEXT.with(|x| {
                        let task_id = x.executor.current_task_id.clone().into_inner();
                        x.epoll.modify_task(&self.fd, task_id, EventFlags::OUT | EventFlags::ONESHOT)
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
    pub(crate) async fn write(&self, buffer: &[u8]) -> Result<usize> {
        TcpWriteFuture::new(&self.fd, buffer)?.await
    }
}