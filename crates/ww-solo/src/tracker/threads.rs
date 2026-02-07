//! Plot thread tracking.

use rand::Rng;
use rand::rngs::StdRng;
use serde::{Deserialize, Serialize};

/// A plot thread being tracked in the story.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Thread {
    /// Thread name/description.
    pub name: String,
    /// Whether this thread is still active.
    pub active: bool,
}

/// List of tracked plot threads.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreadList {
    threads: Vec<Thread>,
}

impl ThreadList {
    /// Create an empty thread list.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a new active thread.
    pub fn add(&mut self, name: impl Into<String>) {
        self.threads.push(Thread {
            name: name.into(),
            active: true,
        });
    }

    /// Close a thread by name. Returns true if found.
    pub fn close(&mut self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        for t in &mut self.threads {
            if t.active && t.name.to_lowercase() == name_lower {
                t.active = false;
                return true;
            }
        }
        false
    }

    /// Remove a thread entirely by name. Returns true if found.
    pub fn remove(&mut self, name: &str) -> bool {
        let name_lower = name.to_lowercase();
        let len_before = self.threads.len();
        self.threads.retain(|t| t.name.to_lowercase() != name_lower);
        self.threads.len() < len_before
    }

    /// Get all active threads.
    pub fn active(&self) -> Vec<&Thread> {
        self.threads.iter().filter(|t| t.active).collect()
    }

    /// Pick a random active thread.
    pub fn random_active(&self, rng: &mut StdRng) -> Option<&Thread> {
        let active: Vec<_> = self.active();
        if active.is_empty() {
            return None;
        }
        Some(active[rng.random_range(0..active.len())])
    }

    /// Get all threads (active and closed).
    pub fn all(&self) -> &[Thread] {
        &self.threads
    }

    /// Number of active threads.
    pub fn active_count(&self) -> usize {
        self.threads.iter().filter(|t| t.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn add_and_list() {
        let mut tl = ThreadList::new();
        tl.add("Find the artifact");
        tl.add("Rescue the prisoner");
        assert_eq!(tl.active_count(), 2);
        assert_eq!(tl.all().len(), 2);
    }

    #[test]
    fn close_thread() {
        let mut tl = ThreadList::new();
        tl.add("Find the artifact");
        assert!(tl.close("Find the artifact"));
        assert_eq!(tl.active_count(), 0);
        assert_eq!(tl.all().len(), 1); // still in list, just inactive
    }

    #[test]
    fn close_case_insensitive() {
        let mut tl = ThreadList::new();
        tl.add("Find the artifact");
        assert!(tl.close("find the artifact"));
        assert_eq!(tl.active_count(), 0);
    }

    #[test]
    fn close_nonexistent() {
        let mut tl = ThreadList::new();
        tl.add("Find the artifact");
        assert!(!tl.close("nonexistent"));
    }

    #[test]
    fn remove_thread() {
        let mut tl = ThreadList::new();
        tl.add("Find the artifact");
        tl.add("Rescue prisoner");
        assert!(tl.remove("Find the artifact"));
        assert_eq!(tl.all().len(), 1);
        assert_eq!(tl.active_count(), 1);
    }

    #[test]
    fn random_active_thread() {
        let mut tl = ThreadList::new();
        tl.add("Thread A");
        tl.add("Thread B");
        let mut rng = StdRng::seed_from_u64(42);
        let t = tl.random_active(&mut rng).unwrap();
        assert!(t.name == "Thread A" || t.name == "Thread B");
    }

    #[test]
    fn random_active_empty() {
        let tl = ThreadList::new();
        let mut rng = StdRng::seed_from_u64(42);
        assert!(tl.random_active(&mut rng).is_none());
    }

    #[test]
    fn serde_roundtrip() {
        let mut tl = ThreadList::new();
        tl.add("Quest");
        tl.close("Quest");
        let json = serde_json::to_string(&tl).unwrap();
        let tl2: ThreadList = serde_json::from_str(&json).unwrap();
        assert_eq!(tl2.all().len(), 1);
        assert_eq!(tl2.active_count(), 0);
    }
}
