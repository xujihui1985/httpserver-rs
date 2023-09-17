use polling::{AsRawSource, AsSource, Event, Poller};

use super::event_handler::EventHandler;

pub struct Reactor {
    pub poller: Poller,
    pub register: Vec<(usize, Box<dyn EventHandler + Sync + Send + 'static>)>,
    pub unregister: Vec<usize>,
    pub tasks: Vec<usize>,
}

impl Reactor {
    pub fn new() -> std::io::Result<Self> {
        let poller = Poller::new()?;
        Ok(Reactor {
            poller,
            tasks: Vec::new(),
            register: Vec::new(),
            unregister: Vec::new(),
        })
    }

    pub fn add(
        &self, 
        source: impl AsRawSource, 
        event: Event,
    ) -> std::io::Result<()> {
        unsafe { self.poller.add(source, event) }
    }

    pub fn schedule(&mut self, id: usize) {
        self.tasks.push(id);
    }

    pub fn remove(&mut self, id: usize, souce: impl AsSource) -> std::io::Result<()> {
        self.poller.delete(souce)?;
        self.unregister.push(id);
        self.schedule(id);
        Ok(())
    }

    pub fn register<T>(&mut self, id: usize, client: T) 
    where T: EventHandler + Sync + Send + 'static {
        self.schedule(id);
        self.register.push((id, Box::new(client)));
    }

    pub fn modify(&mut self, souce: impl AsSource, event: Event) -> std::io::Result<()> {
        self.poller.modify(souce, event)
    }

    pub fn notify(&self) {
        self.poller.notify().unwrap();
    }
}
