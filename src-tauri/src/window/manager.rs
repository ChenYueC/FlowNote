use std::collections::HashMap;

pub struct FloatingWindowManager {
    active_windows: HashMap<String, FloatingWindowHandle>,
}

pub struct FloatingWindowHandle {
    pub window_id: String,
    pub note_id: String,
    pub label: String,
}

impl FloatingWindowManager {
    pub fn new() -> Self {
        Self {
            active_windows: HashMap::new(),
        }
    }

    pub fn register(&mut self, window_id: String, note_id: String, label: String) {
        self.active_windows.insert(
            window_id.clone(),
            FloatingWindowHandle {
                window_id,
                note_id,
                label,
            },
        );
    }

    pub fn unregister(&mut self, window_id: &str) {
        self.active_windows.remove(window_id);
    }

    pub fn get(&self, window_id: &str) -> Option<&FloatingWindowHandle> {
        self.active_windows.get(window_id)
    }

    pub fn list(&self) -> Vec<&FloatingWindowHandle> {
        self.active_windows.values().collect()
    }

    pub fn count(&self) -> usize {
        self.active_windows.len()
    }
}
