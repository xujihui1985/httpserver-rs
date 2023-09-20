use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::mpsc::{Sender, Receiver, self};

pub type LocalBoxedFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

pub struct TaskQueue {
    sender: Sender<Rc<Task>>,
    receiver: Receiver<Rc<Task>>,
    tasks: Vec<Rc<Task>>
}

pub struct Task {
    pub future: RefCell<LocalBoxedFuture<'static, ()>>
}

impl TaskQueue {
    pub fn new() -> Self {
        let (s, r) = mpsc::channel();
        TaskQueue {
            sender: s,
            receiver: r,
            tasks: Vec::new()
        }
    }

    pub fn sender(&self) -> Sender<Rc<Task>> {
        self.sender.clone()
    }

    pub fn receiver(&mut self) {
        // try_iter will never block
        for task in self.receiver.try_iter() {
            self.tasks.push(task);
        }
    }

    pub fn push(&mut self, runnable: Task) {
        self.tasks.push(Rc::new(runnable));
    }

    pub fn pop(&mut self) -> Option<Rc<Task>> {
        self.tasks.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }

}