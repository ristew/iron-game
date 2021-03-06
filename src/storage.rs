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
    Polity,
    Character,
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
        } else if TypeId::of::<T>() == TypeId::of::<Polity>() {
            Self::Polity
        } else if TypeId::of::<T>() == TypeId::of::<Character>() {
            Self::Character
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
    fn new_id(&mut self) -> usize;
    fn get_id(&self, id_num: usize) -> Self::Id;
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

pub struct ObjectStorage<Id>
where
    Id: IronId
{
    id_ctr: usize,
    pub id_map: HashMap<usize, Id>,
}

impl<Id> ObjectStorage<Id>
where
    Id: IronId + Clone,
{
    pub fn has_id(&self, id: &Id) -> bool {
        self.id_map.contains_key(&id.num())
    }

    fn insert_id(&mut self, id: Id) {
        self.id_map
            .insert(id.num(), id);
    }
}

impl<Id> Storage for ObjectStorage<Id>
where
    Id: IronId + Debug + Clone,
{
    type Object = Id::Target;
    type Id = Id;

    fn new() -> Self {
        Self::default()
    }
    fn insert(&mut self, mut data: Id::Target) -> Id {
        let id_num = self.new_id();
        data.set_id(id_num);
        let rc = Rc::new(RefCell::new(data));
        let id = Self::Id::new(id_num, IronIdInner(rc.clone()));
        self.insert_id(id.clone());
        id
    }

    fn new_id(&mut self) -> usize {
        self.id_ctr += 1;
        self.id_ctr
    }

    fn get_id(&self, id_num: usize) -> Id {
        self.id_map.get(&id_num).unwrap().clone()
    }

    fn remove(&mut self, id: &Self::Id) {
        self.id_map.remove(&id.num());
    }
}

impl<Id> Default for ObjectStorage<Id>
where
    Id: IronId,
{
    fn default() -> Self {
        Self {
            id_ctr: 0,
            id_map: HashMap::new(),
        }
    }
}

pub struct Storages {
    storages: HashMap<StorageType, Box<dyn Any>>,
}

impl Storages {
    pub fn get_storage<T>(&self) -> &ObjectStorage<T::IdType>
    where
        T: IronData + 'static,
    {
        self.storages
            .get(&StorageType::match_type::<T>())
            .unwrap()
            .downcast_ref::<ObjectStorage<T::IdType>>()
            .unwrap()
    }
    pub fn get_storage_mut<T>(&mut self) -> &mut ObjectStorage<T::IdType>
    where
        T: IronData + 'static,
    {
        self.storages
            .get_mut(&StorageType::match_type::<T>())
            .unwrap()
            .downcast_mut::<ObjectStorage<T::IdType>>()
            .unwrap()
    }
    // pub fn get_ref<T>(&self, id: &T::IdType) -> IronIdInner<T>
    // where
    //     T: IronData + 'static,
    // {
    //     self.get_storage::<T>().get_ref(id)
    // }
    pub fn insert<T>(&mut self, data: T) -> T::IdType
    where
        T: IronData + 'static,
    {
        self.get_storage_mut::<T>().insert(data)
    }

    pub fn remove<T>(&mut self, id: &T::IdType)
    where
        T: IronData + 'static,
    {
        self.get_storage_mut::<T>().remove(id);

    }

    pub fn new_id<T>(&mut self) -> usize
    where
        T: IronData + 'static,
    {
        self.get_storage_mut::<T>().new_id()
    }

    pub fn get_id<T>(&self, id_num: usize) -> T::IdType
    where
        T: IronData + 'static
    {
        self.get_storage::<T>().get_id(id_num)
    }
}

impl Default for Storages {
    fn default() -> Self {
        let mut storages: HashMap<StorageType, Box<dyn Any>> = HashMap::new();
        macro_rules! init_storage {
            ( $typ:ident ) => {
                storages.insert(
                    StorageType::$typ,
                    Box::new(ObjectStorage::<<$typ as IronData>::IdType>::new()),
                );
            }
        }
        init_storage!(Province);
        init_storage!(Pop);
        init_storage!(Settlement);
        init_storage!(Culture);
        init_storage!(Religion);
        init_storage!(Language);
        init_storage!(Polity);
        init_storage!(Character);
        Self { storages }
    }
}
