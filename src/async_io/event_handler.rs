use polling::Event;

pub trait EventHandler {
    fn id(&self) -> usize;
    fn name(&self) -> String;
    fn event(&mut self, event: Event);

    fn matches(&self, event: Event) -> bool {
        self.id() == event.key
    }

    fn poll(&mut self);
}