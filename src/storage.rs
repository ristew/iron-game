use std::cell::RefCell;
use std::collections::HashMap;
use std::{fmt::Debug, hash::Hash, rc::{Rc, Weak}};

use crate::*;

pub struct Storage<T, Id> where T: IronData, Id: Eq + Hash + Debug + IronId<Target = T> {
    id_ctr: usize,
    pub rcs: Vec<Rc<RefCell<T>>>,
    pub id_map: HashMap<Id, Weak<RefCell<T>>>,
}

impl<T, Id> Storage<T, Id> where T: IronData<IdType = Id>, Id: Eq + Hash + Debug + IronId<Target = T> {
    pub fn insert(&mut self, item: T) -> Weak<RefCell<T>> {
        let rc = Rc::new(RefCell::new(item));
        self.rcs.push(rc.clone());
        self.id_map.insert((*rc).borrow().id(), Rc::downgrade(&rc));
        Rc::downgrade(&rc)
    }

    pub fn get_id(&mut self) -> Id {
        self.id_ctr += 1;
        Id::new(self.id_ctr)
    }

    pub fn get_ref(&self, id: &Id) -> Rc<RefCell<T>> {
        if let Some(rc) = id.try_borrow() {
            rc.clone()
        } else {
            let rc = self.id_map.get(&id).unwrap().upgrade().unwrap();
            id.set_reference(rc.clone());
            rc
        }
        // if id.1.borrow().is_none() {
        // } else {

        // }
    }

    pub fn remove(&mut self, id: &Id) {
        self.id_map.remove(id);
        for removed in self.rcs.drain_filter(|item| item.borrow().id() == *id) {
            println!("removed item: {:?}", removed.borrow().id());
        }
    }
}

impl<T, Id> Default for Storage<T, Id> where T: IronData, Id: Eq + Hash + Debug + IronId<Target = T> {
    fn default() -> Self {
        Self {
            id_ctr: 0,
            rcs: Vec::new(),
            id_map: HashMap::new(),
        }
    }
}
