/// This pattern it has multiple similarities with Event sourcing pattern. \
/// The main idea is to keep track of all actions done over a specific data type, and being
/// able to go back in time in any moment to get the stater of that event.

/// Data structure for the event to persist in time.
pub struct Event {
    state: String,
}

/// Implementation of the Event where we can perform several actions:
/// [new] Create the new Event
/// [save_state] create a Memento with the current state
/// [restore_state] using a Memento we restore the state of the event.
impl Event {
    pub fn new(state: String) -> Self {
        Event { state }
    }

    pub fn save_state(&self) -> Memento {
        Memento::new(self.state.clone())
    }

    pub fn restore_state(&mut self, memento: Memento) {
        self.state = memento.get_state();
    }
}

/// Data type to keep the states of [Event] type
#[derive(Clone)]
pub struct Memento {
    state: String,
}

/// Implementation of the Memento where we can perform actions for:
/// [new] Create a new Memento instance by a passed event state.
/// [get_state] return the state persisted in the Memento type.
impl Memento {
    pub fn new(state: String) -> Self {
        Memento { state }
    }

    pub fn get_state(&self) -> String {
        self.state.clone()
    }
}

/// Data type to emulate an Event sourcing pattern. Where we will persist in memory all previous states
/// of [Event] type
/// We create a [EventSourcing] type per [Event] type.
/// The mementos Vector it will persist all previous states.
pub struct EventSourcing {
    mementos: Vec<Memento>,
}

/// Implementation of EventSourcing with next operators:
/// [new] Create the instance.
/// [add_memento] add a Memento instance in the collection of states.
/// [get_memento] retrieve a specific memento element from the collection.
impl EventSourcing {
    pub fn new() -> Self {
        EventSourcing { mementos: Vec::new() }
    }

    pub fn add_memento(&mut self, memento: Memento) {
        self.mementos.push(memento);
    }

    pub fn get_memento(&self, index: usize) -> Option<Memento> {
        self.mementos.get(index).cloned()
    }
}

#[cfg(test)]
mod tests {
    use crate::behavioral::memento::{Event, EventSourcing};

    #[test]
    fn memento_pattern() {
        let mut events = EventSourcing::new();
        let mut event = Event::new("Hello".to_string());
        println!("${}",event.state);
        events.add_memento(event.save_state());
        event.state = "Hello world".to_string();
        println!("${}",event.state);
        events.add_memento(event.save_state());
        event.state = "Hello world!!!".to_string();
        println!("${}",event.state);
        events.add_memento(event.save_state());
        // Restore state
        let memento = events.get_memento(0).unwrap();
        event.restore_state(memento);
        println!("${}",event.state)
    }
}
