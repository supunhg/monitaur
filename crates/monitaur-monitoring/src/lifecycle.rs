// Container lifecycle tracking (start/stop/restart events)

#[derive(Default)]
pub struct LifecycleTracker;

impl LifecycleTracker {
    pub fn new() -> Self {
        Self
    }

    pub fn track(&self) {
        todo!("subscribe to container lifecycle events")
    }
}
