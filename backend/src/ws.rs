use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct Hub {
    inner: Arc<Mutex<HubInner>>,
}

#[derive(Debug)]
struct HubInner {
    next_id: u64,
    clients: HashMap<u64, mpsc::UnboundedSender<String>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HubInner {
                next_id: 1,
                clients: HashMap::new(),
            })),
        }
    }

    pub fn add_client(&self, tx: mpsc::UnboundedSender<String>) -> u64 {
        let mut inner = self.inner.lock().unwrap();
        let id = inner.next_id;
        inner.next_id += 1;
        inner.clients.insert(id, tx);
        id
    }

    pub fn remove_client(&self, id: u64) {
        let mut inner = self.inner.lock().unwrap();
        inner.clients.remove(&id);
    }

    pub fn broadcast_json<T: Serialize>(&self, msg: &T) {
        let Ok(text) = serde_json::to_string(msg) else {
            return;
        };
        let inner = self.inner.lock().unwrap();
        for (_id, tx) in inner.clients.iter() {
            let _ = tx.send(text.clone());
        }
    }
}
