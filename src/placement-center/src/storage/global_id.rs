pub struct GlobalId {}

impl GlobalId {
    pub fn new() -> Self {
        return GlobalId {};
    }

    pub fn generate(&self) {
        self.add_lock();

        self.remove_lock();
    }

    fn add_lock(&self) {}

    fn remove_lock(&self) {}
}
