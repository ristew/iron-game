use std::{any::TypeId, cell::{Ref, RefCell}, collections::{HashMap, hash_map::Values}, fmt::Debug, hash::Hash, marker::PhantomData, rc::Rc};

use anymap::AnyMap;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBatch, Rect, StrokeOptions},
    Context,
};
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Standard, Uniform};
use rayon::prelude::*;
use serde::{Serialize, Deserialize};

use crate::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        (self.day / 30) % 12 + 1
    }

    pub fn year(&self) -> usize {
        self.day / 360 + 1
    }

    pub fn day_of_month(&self) -> usize {
        self.day % 30 + 1
    }

    pub fn age(&self, now: Date) -> usize {
        (now.day - self.day) / 360
    }

    pub fn add_days(self, days: usize) -> Self {
        Date { day: self.day + days }
    }
}

pub fn parse_path(path: &'static str) {
    let path_regex = r"((self\.)?\w+)\.(.*)";
}

impl Debug for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{:02}/{:02}/{}", self.month(), self.day_of_month(), self.year()).as_str())
    }
}

#[derive(Serialize, Deserialize)]
pub struct World {
    pub date: Date,
    pub province_coord_map: HashMap<Coordinate, ProvinceId>,
    pub storages: Storages,
    #[serde(skip)]
    pub formula_system: FormulaSystem<GameId, FactorType>,
    #[serde(skip)]
    pub commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
    #[serde(skip)]
    pub camera: Camera,
    #[serde(skip)]
    pub events: Events,
    #[serde(skip)]
    pub logs: Logs,
    pub selected_province: Option<ProvinceId>,
    pub population: isize,
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
            for command in event.map_event(self).into_iter() {
                self.add_command(command);
            }
            if event.kind().should_log() {
                println!("[{:?}] {}", self.date, event.short_description(self));
            }
            self.logs.add_log(self.date, event.clone());
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
            .settlement = Some(set_id.clone());
        set_id
    }

    pub fn insert<T>(&mut self, data: T) -> T::IdType
    where
        T: IronData + 'static,
    {
        self.storages.insert(data)
    }

    pub fn remove<Id>(&mut self, id: &Id)
    where
        Id: IronId + 'static,
    {
        self.storages.remove::<Id::Target>(id);
    }

    pub fn new(ctx: &mut Context) -> Self {
        Self {
            date: Date { day: 0 },
            province_coord_map: Default::default(),
            storages: Default::default(),
            formula_system: Default::default(),
            commands: Rc::new(RefCell::new(Vec::new())),
            camera: Default::default(),
            events: Default::default(),
            logs: Default::default(),
            selected_province: Default::default(),
            population: 0,
            // ui_system: Default::default(),
        }
    }

    pub fn iter_storage<T>(&self) -> Values<'_, usize, T::IdType> where T: IronData + 'static {
        self.storages.get_storage::<T>().id_map.values()
    }

    pub fn pixel_to_province(&self, pixel: Point2) -> Option<ProvinceId> {
        let coord = Coordinate::from_pixel_pos(pixel, &self.camera);
        self.get_province_coordinate(coord)
    }
}

fn random_place_name(culture: CultureId) -> String {
    culture.get().language.get().generate_name(3)
}

pub fn pops_yearly_growth(world: &World) {
    for pop in world.iter_storage::<Pop>() {
        // println!("pop size: {}", pop_rc.borrow().size);
        let babies = positive_isample(2, pop.get().size * 4 / 100);
        let deaths = positive_isample(2, pop.get().size / 50);
        // println!("babies {} deaths {} size {}", babies, deaths, pop_rc.borrow().size);
        world.add_command(Box::new(PopGrowthCommand {
            babies,
            deaths,
            pop: pop.clone(),
        }));
    }
    world.add_command(Box::new(UpdateWorldPopulation));
}

pub fn harvest_provinces(world: &World) {
    for province in world.iter_storage::<Province>() {
        if world.date.month() == province.get().harvest_month {
            if let Some(settlement) = &province.get().settlement {
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
        for character in world.iter_storage::<Character>() {
            if character.get().death.is_none() && character.get().birthday.age(world.date) as f32 > character.get().health {
                // sic fortuna
                world.events.add_deferred(
                    Rc::new(CharacterDiedEvent(character.clone())),
                    world.date.day + Uniform::from(1..360).sample(&mut thread_rng()),
                );
            }
        }
    }

    if world.date.is_month() {
        harvest_provinces(world);
        for pop in world.iter_storage::<Pop>() {
            world.add_command(Box::new(PopEatCommand(pop.clone())));
        }
    }
}
