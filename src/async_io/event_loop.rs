use std::{sync::{Arc, RwLock}, hash::Hash, collections::HashMap};

use polling::Events;

use super::{reactor::Reactor, event_handler::EventHandler};

pub struct EventLoop {
    reactor: Arc<RwLock<Reactor>>,
    pub sources: HashMap<usize, Box<dyn EventHandler>>,
}

impl EventLoop {
    pub fn new(reactor: Arc<RwLock<Reactor>>) -> Self {
        Self { 
            reactor,
            sources: HashMap::new(),
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        loop {
            loop {

                // 1. process tasks
                if let Some(id) = {
                    let mut reactor = self.reactor.write().unwrap();
                    reactor.tasks.pop()
                } {
                    if let Some(source) = self.sources.get_mut(&id) {
                        source.poll();
                    }
                }

                // 2. register new sources
                self.handle_register();

                if self.reactor.read().unwrap().tasks.is_empty() {
                    break;
                }
            }

            // 3. unregister dropped sources
            self.handle_unregister();

            // wait for io
            self.wait_for_io();
        }
    }

    fn wait_for_io(&mut self) -> std::io::Result<()> {
        let mut events = Events::new();
        {
            let reactor = self.reactor.read().unwrap();
            reactor.poller.wait(&mut events, None)?;
        }
        for ev in events.iter() {
            if let Some(source) = self.sources.get_mut(&ev.key) {
                source.event(ev);
            }

        }
        Ok(())
    }

    fn handle_register(&mut self) {
        let mut reactor = self.reactor.write().unwrap();
        while let Some((id, source)) = reactor.register.pop() {
            self.sources.insert(id, source);
        }
    }

    fn handle_unregister(&mut self) {
        let mut reactor = self.reactor.write().unwrap();
        while let Some(id) = reactor.unregister.pop() {
            self.sources.remove(&id);
        }

    }

}
