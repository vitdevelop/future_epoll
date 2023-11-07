mod accept;
mod read;
mod write;

use std::net::SocketAddr;
use std::os::fd::OwnedFd;
use std::str::FromStr;
use log::error;
use rustix::event::epoll::EventFlags;
use rustix::net::{AddressFamily, bind, listen, socket_with, SocketAddrAny, SocketFlags, SocketType};
use rustix::net::sockopt::{set_socket_reuseaddr, set_socket_reuseport};
use crate::context::CONTEXT;
use crate::Result;

pub(crate) struct TcpServer {
    fd: OwnedFd,
    _address: SocketAddrAny,
}

pub(crate) struct TcpClient {
    fd: OwnedFd,
    _address: SocketAddrAny,
}

impl TcpServer {
    pub(crate) fn listen(addr: String) -> Result<TcpServer> {
        let fd = socket_with(AddressFamily::INET, SocketType::STREAM,
                             SocketFlags::CLOEXEC | SocketFlags::NONBLOCK, None)?;

        set_socket_reuseaddr(&fd, true)?;
        set_socket_reuseport(&fd, true)?;

        let socket_addr = SocketAddr::from_str(addr.as_str())?;
        bind(&fd, &socket_addr)?;

        listen(&fd, 0)?;

        CONTEXT.with(|x| x.epoll.add(&fd, EventFlags::IN))?;

        Ok(TcpServer { fd, _address: SocketAddrAny::from(socket_addr) })
    }
}

impl Drop for TcpClient {
    fn drop(&mut self) {
        CONTEXT.with(|x| {
            if let Err(e) = x.epoll.remove(&self.fd) {
                error!("Unable to remove client socket from epoll, {:?}", e)
            }
        });
    }
}