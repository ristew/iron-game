use std::{hash::Hash, collections::HashMap, sync::Arc};

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};

use crate::*;


pub struct Log {
    event: Rc<dyn Event>,
}

impl Log {
    pub fn new(event: Rc<dyn Event>) -> Self {
        Self {
            event,
        }
    }
}

pub fn push_map<K: Eq + Hash>(map: &DashMap<K, Vec<LogId>>, key: K, log_id: LogId) {
    map
        .entry(key)
        .or_default()
        .push(log_id);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogId(usize);

pub struct Logs {
    storage: Vec<Log>,
    date_map: DashMap<Date, Vec<LogId>>,
    subject_map: DashMap<GameId, Vec<LogId>>,
}

impl Logs {
    pub fn add_log(&mut self, date: Date, event: Rc<dyn Event>) {
        let subjects = event.subjects();
        let log_id = LogId(self.storage.len());
        // println!("{:?}", log_id);
        self.storage.push(Log::new(event));
        push_map(&self.date_map, date, log_id);
        for &subject in subjects.iter() {
            push_map(&self.subject_map, subject, log_id)
        }
    }

    pub fn search_logs<T>(&self, id: T) -> Vec<LogId> where T: IronId {
        self.search_logs_gid(id.gid())
    }

    pub fn search_logs_gid(&self, gid: GameId) -> Vec<LogId> {
        self.subject_map.get(&gid).map(|rr| rr.clone()).unwrap_or_default()
    }

    pub fn get_log<'a>(&'a self, lid: LogId) -> &'a Log {
        &self.storage[lid.0]
    }
}

impl Default for Logs {
    fn default() -> Self {
        Self { storage: Default::default(), date_map: Default::default(), subject_map: Default::default() }
    }
}
