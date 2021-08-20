use std::{
    any::TypeId,
    cell::{Ref, RefCell},
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    rc::Rc,
};

use anymap::AnyMap;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBatch, Rect, StrokeOptions},
    Context,
};
use rand::{thread_rng, Rng};
use rand_distr::{Standard, Uniform};
use rayon::prelude::*;

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub day: usize,
}

impl Date {
    pub fn is_month(&self) -> bool {
        self.day % 30 == 0
    }

    pub fn is_year(&self) -> bool {
        self.day % 360 == 0
    }

    pub fn month(&self) -> usize {
        (self.day / 30) % 12
    }

    pub fn year(&self) -> usize {
        self.day / 360 + 1
    }

    pub fn day_of_month(&self) -> usize {
        self.day % 30 + 1
    }
}

pub fn parse_path(path: &'static str) {
    let path_regex = r"((self\.)?\w+)\.(.*)";
}

impl Debug for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}/{}/{}", self.month(), self.day_of_month(), self.year()).as_str())
    }
}

pub struct World {
    pub date: Date,
    pub province_coord_map: HashMap<Coordinate, ProvinceId>,
    pub storages: Storages,
    pub commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
    pub camera: Camera,
    pub events: Events,
    pub selected_province: Option<ProvinceId>,
}

impl World {
    pub fn add_command(&self, command: Box<dyn Command>) {
        self.commands.borrow_mut().push(command);
    }

    pub fn process_command_queue(&mut self) {
        let commands = self.commands.replace(Vec::new());
        for command in commands {
            command.run(self);
        }
    }

    pub fn process_events(&mut self) {
        self.events.update(self);
        let events = self.events.events.replace(Vec::new());
        for event in events {
            if let Some(command) = event.map_event(self) {
                self.add_command(command);
            }
        }
    }

    pub fn insert_province(&mut self, province: Province) {
        let province_id = self.insert::<Province>(province);
        self.province_coord_map
            .insert(province_id.get().coordinate, province_id.clone());
    }

    pub fn get_province_coordinate(&self, coord: Coordinate) -> Option<ProvinceId> {
        self.province_coord_map.get(&coord).map(|p| p.clone())
    }

    pub fn insert_settlement(&mut self, settlement: Settlement) -> SettlementId {
        let set_id = self.insert::<Settlement>(settlement);
        set_id
            .get()
            .province
            .get_mut()
            .settlements
            .push(set_id.clone());
        set_id
    }

    // pub fn get_ref<T>(&self, id: &T::IdType) -> IronIdInner<T>
    // where
    //     T: IronData + 'static,
    // {
    //     self.storages.get_ref::<T>(id)
    // }

    // pub fn map_borrow<T, F, V>(&self, id: &T::IdType, f: F) -> &V where F: Fn(&T) -> &V, T: IronData + 'static {
    //     let refcell = self.get_ref::<T>(id);
    //     f(refcell.into_inner())

    // }

    pub fn insert<T>(&mut self, data: T) -> T::IdType
    where
        T: IronData + 'static,
    {
        // let id = self.storages.get
        self.storages.insert(data)
    }

    pub fn new(ctx: &mut Context) -> Self {
        Self {
            date: Date { day: 0 },
            province_coord_map: Default::default(),
            storages: Default::default(),
            commands: Rc::new(RefCell::new(Vec::new())),
            camera: Default::default(),
            events: Default::default(),
            selected_province: Default::default(),
            // ui_system: Default::default(),
        }
    }

    pub fn pixel_to_province(&self, pixel: Point2) -> Option<ProvinceId> {
        let coord = Coordinate::from_pixel_pos(pixel, &self.camera);
        self.get_province_coordinate(coord)
    }
}

fn random_place_name(culture: CultureId, world: &World) -> String {
    let mut name = String::new();

    name
}

pub fn pops_yearly_growth(world: &World) {
    for pop_ref in world.storages.get_storage::<Pop>().id_map.values() {
        let pop_rc = pop_ref.upgrade().unwrap();
        // println!("pop size: {}", pop_rc.borrow().size);
        let babies = positive_isample(2, pop_rc.borrow().size * 4 / 100);
        let deaths = positive_isample(2, pop_rc.borrow().size / 50);
        world.add_command(Box::new(PopGrowthCommand {
            babies,
            deaths,
            pop: pop_rc.borrow().id().clone(),
        }));
    }
}

pub fn harvest_provinces(world: &World) {
    for province in world.storages.get_storage::<Province>().rcs.iter() {
        if world.date.month() == province.borrow().harvest_month {
            for settlement in province.borrow().settlements.iter() {
                for pop in settlement.get().pops.iter() {
                    harvest(pop, world);
                }
            }
        }
    }
}

pub fn day_tick(world: &World) {
    if world.date.is_year() {
        pops_yearly_growth(world);
    }

    if world.date.is_month() {
        harvest_provinces(world);
        for pop in world.storages.get_storage::<Pop>().rcs.iter() {
            world.add_command(Box::new(PopEatCommand(pop.borrow().id().clone())));
        }
    }
}
