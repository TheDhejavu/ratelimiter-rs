use std::collections::HashMap;
use std::sync::{Mutex, Arc};

#[derive(Clone)]
pub(crate) enum Storage {
    InMemory(Arc<Mutex<HashMap<String, Vec<u64>>>>),
    Redis(redis::Client),
}
