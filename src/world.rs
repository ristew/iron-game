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

macro_rules! object {
	( ex:expr p:expr ) => {
        let path = parse_path(stringify!{$ex})
	};
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
        self.events.update();
        let events = self.events.events.replace(Vec::new());
        for event in events {
            if let Some(command) = event.map_event(self) {
                self.add_command(command);
            }
        }
    }

    pub fn insert_province(&mut self, province: Province) {
        self.province_coord_map
            .insert(province.coordinate, province.id.clone());
        self.insert::<Province>(province);
    }

    pub fn get_province_coordinate(&self, coord: Coordinate) -> Option<ProvinceId> {
        self.province_coord_map.get(&coord).map(|p| p.clone())
    }

    pub fn insert_settlement(&mut self, settlement: Settlement) {
        settlement
            .province
            .get(self)
            .borrow_mut()
            .settlements
            .push(settlement.id.clone());
        self.insert::<Settlement>(settlement);
    }

    pub fn get_ref<T>(&self, id: &T::IdType) -> Rc<RefCell<T>>
    where
        T: IronData + 'static,
    {
        self.storages.get_ref::<T>(id)
    }

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

fn add_test_settlement(world: &mut World, culture_id: CultureId, province_id: ProvinceId) {
    let settlement_id = world.storages.get_id::<Settlement>();
    let pop_id = world.storages.get_id::<Pop>();

    let pop = world.insert(Pop {
        id: pop_id.clone(),
        size: 100,
        farmed_good: Some(Wheat),
        culture: culture_id.clone(),
        settlement: settlement_id.clone(),
        province: province_id.clone(),
        satiety: Satiety {
            base: 0.0,
            luxury: 0.0,
        },
        kid_buffer: KidBuffer::new(),
        owned_goods: GoodStorage(HashMap::new()),
    });

    world
        .get_ref::<Pop>(&pop)
        .borrow_mut()
        .owned_goods
        .add(Wheat, 30000.0);

    let name = culture_id.get(world).borrow().language.get(world).borrow().generate_name();
    world.insert_settlement(Settlement {
        id: settlement_id.clone(),
        name,
        pops: vec![pop_id.clone()],
        features: Vec::new(),
        primary_culture: culture_id.clone(),
        province: province_id.clone(),
        level: SettlementLevel::Village,
    });
}

fn random_place_name(culture: CultureId, world: &World) -> String {
    let mut name = String::new();

    name
}

pub fn create_test_world(world: &mut World) {
    let culture_id = world.storages.get_id::<Culture>();
    let religion_id = world.storages.get_id::<Religion>();
    let language_id = world.storages.get_id::<Language>();

    world.insert(Religion {
        id: religion_id.clone(),
        name: "Test Religion".to_owned(),
    });

    let mut language = Language::new(language_id.clone());
    world.insert(Culture {
        id: culture_id.clone(),
        language: language_id.clone(),
        name: language.generate_name(),
        religion: religion_id.clone(),
        features: Vec::new(),
    });
    language.name = language.generate_name();
    world.insert(language);

    // create provinces
    for i in 0..100 {
        for j in 0..100 {
            let province_id = world.storages.get_id::<Province>();
            let coordinate = Coordinate::new(i - (j / 2), j);
            world.insert_province(Province {
                id: province_id.clone(),
                terrain: Terrain::Hills,
                climate: Climate::Mild,
                coordinate,
                harvest_month: 8,
                settlements: Vec::new(),
            });

            for i in 0..thread_rng().sample(Uniform::new(0, 5)) {
                add_test_settlement(world, culture_id.clone(), province_id.clone());
            }
        }
    }
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
            pop: pop_rc.borrow().id.clone(),
        }));
    }
}

pub fn harvest_provinces(world: &World) {
    for province in world.storages.get_storage::<Province>().rcs.iter() {
        if world.date.month() == province.borrow().harvest_month {
            for settlement in province.borrow().settlements.iter() {
                for pop in world.get_ref::<Settlement>(settlement).borrow().pops.iter() {
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
