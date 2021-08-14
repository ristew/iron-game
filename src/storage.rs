use rayon::iter::IntoParallelIterator;
use rayon::vec::IntoIter;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use std::{
    any::TypeId,
    fmt::Debug,
    hash::Hash,
    rc::{Rc, Weak},
};
use strum::{EnumIter, IntoEnumIterator};

use crate::*;

#[derive(Debug, Copy, Clone, EnumIter, PartialEq, Eq, Hash)]
pub enum StorageType {
    Pop,
    Province,
    Culture,
    Religion,
    Settlement,
    Language,
}

impl StorageType {
    fn match_type<T: 'static>() -> Self {
        if TypeId::of::<T>() == TypeId::of::<Pop>() {
            Self::Pop
        } else if TypeId::of::<T>() == TypeId::of::<Province>() {
            Self::Province
        } else if TypeId::of::<T>() == TypeId::of::<Culture>() {
            Self::Culture
        } else if TypeId::of::<T>() == TypeId::of::<Religion>() {
            Self::Religion
        } else if TypeId::of::<T>() == TypeId::of::<Settlement>() {
            Self::Settlement
        } else if TypeId::of::<T>() == TypeId::of::<Language>() {
            Self::Language
        } else {
            panic!("could not match Id type to storage, {}", stringify! {T});
        }
    }
}

pub trait Storage {
    type Object: IronData<IdType = Self::Id>;
    type Id: IronId<Target = Self::Object>;

    fn new() -> Self
    where
        Self: Sized;
    fn insert(&mut self, item: Self::Object) -> Self::Id;
    fn get_id(&mut self) -> Self::Id;
    fn get_ref(&self, id: &Self::Id) -> Rc<RefCell<Self::Object>>;
    fn remove(&mut self, id: &Self::Id);
}

// pub struct InnerStorage<T>(Vec<Arc<T>>);

// impl<T> IntoParallelIterator for InnerStorage<T> where T: Send + std::marker::Sync {
//     type Iter = IntoIter<Arc<T>>;

//     type Item = Arc<T>;

//     fn into_par_iter(self) -> Self::Iter {
//         IntoIter { vec: self.0 }
//     }
// }

pub struct ObjectStorage<T, Id>
where
    T: IronData,
{
    id_ctr: usize,
    pub rcs: Vec<Rc<RefCell<T>>>,
    pub id_map: HashMap<usize, Weak<RefCell<T>>>,
    _fake: PhantomData<Id>,
}

impl<T, Id> ObjectStorage<T, Id>
where
    T: IronData<IdType = Id>,
    Id: IronId<Target = T>,
{
    fn load_id(&self, id: &T::IdType) {
        let rc = self.id_map.get(&id.num()).unwrap().upgrade().unwrap();
        id.set_reference(rc.clone());
    }

    fn get_ref_impl(&self, id: &Id) -> Rc<RefCell<T>> {
        if let Some(rc) = id.try_borrow() {
            rc.clone()
        } else {
            let rc = self.id_map.get(&id.num()).unwrap().upgrade().unwrap();

            id.set_reference(rc.clone());
            rc.clone()
        }
        // if id.1.borrow().is_none() {
        // } else {

        // }
    }
}

impl<T, Id> Storage for ObjectStorage<T, Id>
where
    T: IronData<IdType = Id>,
    Id: IronId<Target = T> + Debug,
{
    type Object = T;
    type Id = Id;

    fn new() -> Self {
        Self::default()
    }
    fn insert(&mut self, item: Self::Object) -> Self::Id {
        let rc = Rc::new(RefCell::new(item));
        self.rcs.push(rc.clone());
        self.id_map
            .insert((*rc).borrow().id().num(), Rc::downgrade(&rc));
        let x = (*rc).borrow();
        x.id()
    }

    fn get_id(&mut self) -> <T as IronData>::IdType {
        self.id_ctr += 1;
        T::IdType::new(self.id_ctr)
    }

    fn remove(&mut self, id: &T::IdType) {
        self.id_map.remove(&id.num());
        for removed in self
            .rcs
            .drain_filter(|item| item.borrow().id().num() == id.num())
        {
            println!("removed item: {:?}", removed.borrow().id());
        }
    }

    fn get_ref(&self, id: &Id) -> Rc<RefCell<T>> {
        self.get_ref_impl(id)
    }
}

impl<T, Id> Default for ObjectStorage<T, Id>
where
    T: IronData,
{
    fn default() -> Self {
        Self {
            id_ctr: 0,
            rcs: Vec::new(),
            id_map: HashMap::new(),
            _fake: Default::default(),
        }
    }
}

pub struct Storages {
    storages: HashMap<StorageType, Box<dyn Any>>,
}

impl Storages {
    pub fn get_storage<T>(&self) -> &ObjectStorage<T, T::IdType>
    where
        T: IronData + 'static,
    {
        self.storages
            .get(&StorageType::match_type::<T>())
            .unwrap()
            .downcast_ref::<ObjectStorage<T, T::IdType>>()
            .unwrap()
    }
    pub fn get_storage_mut<T>(&mut self) -> &mut ObjectStorage<T, T::IdType>
    where
        T: IronData + 'static,
    {
        self.storages
            .get_mut(&StorageType::match_type::<T>())
            .unwrap()
            .downcast_mut::<ObjectStorage<T, T::IdType>>()
            .unwrap()
    }
    pub fn get_ref<T>(&self, id: &T::IdType) -> Rc<RefCell<T>>
    where
        T: IronData + 'static,
    {
        self.get_storage::<T>().get_ref(id)
    }
    pub fn insert<T>(&mut self, data: T) -> T::IdType
    where
        T: IronData + 'static,
    {
        self.get_storage_mut::<T>().insert(data)
    }
    pub fn get_id<T>(&mut self) -> T::IdType
    where
        T: IronData + 'static,
    {
        self.get_storage_mut::<T>().get_id()
    }
}

impl Default for Storages {
    fn default() -> Self {
        let mut storages: HashMap<StorageType, Box<dyn Any>> = HashMap::new();
        storages.insert(
            StorageType::Province,
            Box::new(ObjectStorage::<Province, ProvinceId>::new()),
        );
        storages.insert(
            StorageType::Pop,
            Box::new(ObjectStorage::<Pop, PopId>::new()),
        );
        storages.insert(
            StorageType::Settlement,
            Box::new(ObjectStorage::<Settlement, SettlementId>::new()),
        );
        storages.insert(
            StorageType::Culture,
            Box::new(ObjectStorage::<Culture, CultureId>::new()),
        );
        storages.insert(
            StorageType::Religion,
            Box::new(ObjectStorage::<Religion, ReligionId>::new()),
        );
        storages.insert(
            StorageType::Language,
            Box::new(ObjectStorage::<Language, LanguageId>::new()),
        );
        Self { storages }
    }
}
