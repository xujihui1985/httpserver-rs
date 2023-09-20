use std::{collections::HashMap, task::{Waker, Context}, time::Duration, cell::RefCell};

use polling::{AsRawSource, Event, Poller, AsSource, Events};

thread_local! {
    pub static REACTOR: RefCell<Reactor> = RefCell::new(Reactor::new());
}

pub struct Reactor {
    readable: HashMap<usize, Vec<Waker>>,
    writeable: HashMap<usize, Vec<Waker>>,
    poller: Poller,
}

impl Reactor {
    pub fn new() -> Self {
        Reactor {
            readable: HashMap::new(),
            writeable: HashMap::new(),
            poller: Poller::new().unwrap(),
        }
    }

    fn get_interest(&self, key: usize) -> Event {
        let readable = self.readable.contains_key(&key);
        let writeable = self.writeable.contains_key(&key);
        match (readable, writeable) {
            (false, false) => Event::none(key),
            (true, false) => Event::readable(key),
            (false, true) => Event::writable(key),
            (true, true) => Event::all(key),
        }
    }

    pub fn add(&self, source: impl AsRawSource) {
        let key = source.raw() as usize;
        unsafe { self.poller.add(source, self.get_interest(key)).unwrap() };
    }

    pub fn remove(&mut self, source: impl AsSource + AsRawSource) {
        let key = source.raw() as usize;
        self.poller.delete(source).unwrap();
        self.readable.remove(&key);
        self.writeable.remove(&key);
    }

    pub fn wake_on_readable(&mut self, source: impl AsRawSource + AsSource, cx: &mut Context) {
        let key = source.raw() as usize;
        self.readable
            .entry(key)
            .or_default()
            .push(cx.waker().clone());
        self.poller.modify(source, self.get_interest(key)).unwrap();
    }
    

    pub fn wake_on_writable(&mut self, source: impl AsRawSource + AsSource, cx: &mut Context) {
        let key = source.raw() as usize;
        self.writeable
            .entry(key)
            .or_default()
            .push(cx.waker().clone());
        self.poller.modify(source, self.get_interest(key)).unwrap();
    }

    pub fn drain_wakers(&mut self, events: Events) -> Vec<Waker> {
        let mut wakers = Vec::new();
        for ev in events.iter() {
            if let Some((_, readers)) = self.readable.remove_entry(&ev.key) {
                wakers.extend(readers);
            }
            if let Some((_, writers)) = self.writeable.remove_entry(&ev.key) {
                wakers.extend(writers);
            }
        }
        wakers
    }

    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> std::io::Result<usize>{
        self.poller.wait(events, timeout)
    }

    pub fn waiting_on_events(&self) -> bool {
        !self.readable.is_empty() || !self.writeable.is_empty()
    }
}
