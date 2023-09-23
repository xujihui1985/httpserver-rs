use std::{
    cell::RefCell,
    future::Future,
    task::{Context, Poll},
};

use polling::Events;

use super::{
    reactor::{self, REACTOR},
    task_queue::{Task, TaskQueue},
    waker_util::waker_fn,
};

thread_local! {
    pub static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::new());
}

pub fn block_on<F>(f: F) -> std::io::Result<()>
where
    F: Future<Output = ()> + 'static,
{
    EXECUTOR.with(|current| -> std::io::Result<()> {
        let execut = current.borrow();
        execut.spawn(f);
        execut.run()
    })
}

pub fn spawn<F>(f: F)
where
    F: Future<Output = ()> + 'static,
{
    EXECUTOR.with(|current| {
        let execut = current.borrow();
        execut.spawn(f);
    })
}

pub struct Executor {
    tasks: RefCell<TaskQueue>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: RefCell::new(TaskQueue::new()),
        }
    }

    pub fn spawn<F>(&self, f: F)
    where
        F: Future<Output = ()> + 'static,
    {
        self.tasks.borrow_mut().push(Task {
            future: RefCell::new(Box::pin(f)),
        });
    }

    pub fn run(&self) -> std::io::Result<()> {
        loop {
            loop {
                if let Some(task) = {
                    let mut tasks = self.tasks.borrow_mut();
                    tasks.pop()
                } {
                    let waker = {
                        let sender = self.tasks.borrow().sender();
                        let waker_task = task.clone();
                        waker_fn(move || {
                            sender.send(waker_task.clone()).unwrap();
                        })
                    };
                    let mut ctx = Context::from_waker(&waker);
                    match task.future.borrow_mut().as_mut().poll(&mut ctx) {
                        Poll::Ready(()) => {}
                        Poll::Pending => {}
                    }
                }
                if self.tasks.borrow().is_empty() {
                    break;
                }
            }

            if !REACTOR.with(|r| r.borrow().waiting_on_events()) {
                break Ok(());
            }
            self.wait_for_io()?;
            self.tasks.borrow_mut().receiver();
        }
    }

    pub fn wait_for_io(&self) -> std::io::Result<()> {
        REACTOR.with(|current| -> std::io::Result<()> {
            let mut events = Events::new();
            {
                let reactor = current.borrow();
                reactor.wait(&mut events, None)?;
            }
            let wakers = {
                let mut reactor = current.borrow_mut();
                reactor.drain_wakers(events)
            };
            for w in wakers {
                w.wake();
            }
            Ok(())
        })
    }
}
