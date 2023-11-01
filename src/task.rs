use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output=T> + 'a>>;

pub(crate) struct Task {
    pub(crate) future: LocalBoxFuture<'static, ()>,

    pub(crate) sender: Rc<RefCell<Vec<Task>>>,
}