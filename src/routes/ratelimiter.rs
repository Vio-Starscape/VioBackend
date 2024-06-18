use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, Duration};

pub struct RequestCount {
    pub counts: Mutex<HashMap<String, Vec<SystemTime>>>,
}

impl RequestCount {
    pub fn increment(&self, key: &str) -> u32 {
        let mut counts = self.counts.lock().unwrap();
        let counter = counts.entry(key.to_string()).or_insert(Vec::new());

        let now = SystemTime::now();
        counter.retain(|&time| now.duration_since(time).unwrap() < Duration::from_secs(60));

        counter.push(now);

        counter.len() as u32
    }
}