use std::future::Future;
use std::net::{IpAddr, SocketAddr};
use std::os::fd::OwnedFd;
use std::pin::Pin;
use std::task::{Context, Poll};
use rustix::event::epoll::EventFlags;
use rustix::io;
use rustix::net::{acceptfrom_with, SocketAddrAny, SocketFlags};
use crate::context::CONTEXT;
use crate::Result;
use crate::tcp::{TcpClient, TcpServer};

pub(crate) struct TcpAcceptFuture<'a> {
    fd: &'a OwnedFd,
}

impl TcpAcceptFuture<'_> {
    pub(super) fn new(fd: &OwnedFd) -> Result<TcpAcceptFuture> {
        Ok(TcpAcceptFuture {
            fd
            // fd: fd.try_clone()? // TODO: increment fd
        })
    }
}

impl Future for TcpAcceptFuture<'_> {
    type Output = Result<(OwnedFd, SocketAddrAny)>;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        return match acceptfrom_with(&self.fd, SocketFlags::NONBLOCK) {
            Ok((client_fd, client_addr)) => {
                let addr = match client_addr {
                    Some(addr) => addr,
                    None => {
                        SocketAddrAny::from(SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 0))
                    }
                };

                CONTEXT.with(|x| x.epoll.remove(&self.fd))?;
                return Poll::Ready(Ok((client_fd, addr)));
            }
            Err(e) => {
                if e == io::Errno::AGAIN || e == io::Errno::WOULDBLOCK {
                    CONTEXT.with(|x| {
                        let task_id = x.executor.current_task_id.clone().into_inner();
                        x.epoll.wait_task(&self.fd, task_id)
                    })?;

                    Poll::Pending
                } else {
                    return Err(e)?;
                }
            }
        };
    }
}

impl TcpServer {
    pub(crate) async fn accept(&self) -> Result<TcpClient> {
        let (client_fd, addr) = TcpAcceptFuture::new(&self.fd)?.await?;
        CONTEXT.with(|x| x.epoll.add(&client_fd, EventFlags::empty()))?;
        Ok(TcpClient {
            fd: client_fd,
            _address: addr,
        })
    }
}