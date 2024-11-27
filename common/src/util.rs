use std::collections::HashMap;

pub struct Pendings<T> {
    commands: HashMap<u64, T>,
    next_id: u64,
}

impl<T> Pendings<T> {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert(&mut self, sender: T) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.commands.insert(id, sender);
        id
    }

    pub fn remove(&mut self, id: u64) -> Option<T> {
        self.commands.remove(&id)
    }
}
