use crate::epoll::Epoll;
use crate::executor::Executor;

thread_local! {
    pub(crate) static CONTEXT: Context<'static> = Context::new();
}

pub(crate) struct Context<'a> {
    pub(crate) executor: Executor<'a>,
    pub(crate) epoll: Epoll,
}

impl Context<'_> {
    fn new() -> Context<'static> {
        let epoll = Epoll::new().unwrap(); // controlled panic if error
        Context {
            executor: Executor::new(),
            epoll,
        }
    }
}