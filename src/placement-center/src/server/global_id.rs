pub struct GlobalId {}

impl GlobalId {
    pub fn new() -> Self {
        return GlobalId {};
    }

    pub fn generate() {
        self.add_lock();

        self.remove_lock();
    }

    fn add_lock() {}

    fn remove_lock() {}
}
