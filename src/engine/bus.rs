use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use super::event::EngineEvent;

pub type EventHandler = Arc<dyn Fn(&EngineEvent) + Send + Sync>;

pub struct EventBus {
    subscribers: Arc<Mutex<HashMap<String, Vec<EventHandler>>>>,
    event_counts: Arc<Mutex<HashMap<String, u64>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            event_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn subscribe<F>(&self, event_type: &str, handler: F) -> Result<(), String>
    where
        F: Fn(&EngineEvent) + Send + Sync + 'static,
    {
        let mut subs = self.subscribers.lock().map_err(|e| e.to_string())?;
        subs.entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(handler));
        Ok(())
    }

    pub fn publish(&self, event: EngineEvent) -> Result<(), String> {
        let event_type = event.event_type().to_string();
        if let Ok(mut counts) = self.event_counts.lock() {
            let counter = counts.entry(event_type.clone()).or_insert(0);
            *counter += 1;
        }
        let subs = self.subscribers.lock().map_err(|e| e.to_string())?;

        if let Some(handlers) = subs.get(&event_type) {
            for handler in handlers {
                handler(&event);
            }
        }

        Ok(())
    }

    pub fn publish_all(&self, event: EngineEvent) -> Result<(), String> {
        let subs = self.subscribers.lock().map_err(|e| e.to_string())?;

        for handlers in subs.values() {
            for handler in handlers {
                handler(&event);
            }
        }

        Ok(())
    }

    pub fn metrics_snapshot(&self) -> HashMap<String, u64> {
        self.event_counts
            .lock()
            .map(|m| m.clone())
            .unwrap_or_default()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
            event_counts: Arc::clone(&self.event_counts),
        }
    }
}