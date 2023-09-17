use std::{thread, sync::{mpsc, Arc, Mutex}};

pub struct Worker {
    id: usize,
    pub(crate) thread: Option<thread::JoinHandle<()>>,
}

pub type Job = Box<dyn FnOnce() -> std::io::Result<()> + Send + 'static>;

pub enum Task {
   New(Job),
   Exit
}


impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Task>>>) -> Self {
        let handle = thread::spawn(move || {
            loop {
                // release the lock after the scope
                let task = {
                    let rx = receiver.lock().unwrap();
                    rx.recv().unwrap()
                };
                match task {
                    Task::New(job) => job().unwrap(),
                    Task::Exit => break
                }
            }
        });
        Worker {
            id: id,
            thread: Some(handle),
        }
    }
}
