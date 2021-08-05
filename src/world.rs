use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use ggez::{Context, graphics::{Color, DrawMode, DrawParam, Mesh, MeshBatch, StrokeOptions}};

use crate::*;

pub struct Date {
    pub day: usize
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

impl Debug for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{}/{}/{}", self.month(), self.day_of_month(), self.year()).as_str())
    }
}


pub struct World {
    pub date: Date,
    pub provinces: Storage<Province, ProvinceId>,
    pub province_coord_map: HashMap<Coordinate, ProvinceId>,
    pub cultures: Storage<Culture, CultureId>,
    pub religions: Storage<Religion, ReligionId>,
    pub settlements: Storage<Settlement, SettlementId>,
    pub pops: Storage<Pop, PopId>,
    pub commands: Rc<RefCell<Vec<Box<dyn Command>>>>,
    pub camera: Camera,
    pub events: Events,
    pub ui_system: UiSystem,
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
        self.province_coord_map.insert(province.coordinate, province.id.clone());
        self.provinces.insert(province);
    }

    pub fn get_province_coordinate(&self, coord: Coordinate) -> ProvinceId {
        self.province_coord_map.get(&coord).unwrap().clone()
    }

    pub fn insert_settlement(&mut self, settlement: Settlement) {
        self.provinces.get_ref(&self.get_province_coordinate(settlement.coordinate))
                      .borrow_mut().settlements.push(settlement.id.clone());
        self.settlements.insert(settlement);
    }

    pub fn new(ctx: &mut Context) -> Self {
        Self {
            date: Date { day: 0 },
            provinces: Default::default(),
            province_coord_map: Default::default(),
            cultures: Default::default(),
            religions: Default::default(),
            settlements: Default::default(),
            pops: Default::default(),
            commands: Rc::new(RefCell::new(Vec::new())),
            camera: Default::default(),
            events: Default::default(),
            ui_system: Default::default(),
        }
    }
}

pub fn create_test_world(world: &mut World) {
    let culture_id = world.cultures.get_id();
    let religion_id = world.religions.get_id();

    world.religions.insert(Religion {
        id: religion_id.clone(),
        name: "Test Religion".to_owned(),
    });

    world.cultures.insert(Culture {
        id: culture_id.clone(),
        name: "Test People".to_owned(),
        religion: religion_id.clone(),
        features: Vec::new(),
    });
    // create provinces
    for i in 0..25 {
        for j in 0..25 {
            let province_id = world.provinces.get_id();
            let coordinate = Coordinate::new(i, j);
            world.insert_province(Province {
                id: province_id,
                terrain: Terrain::Hills,
                climate: Climate::Mild,
                coordinate,
                harvest_month: 8,
                settlements: Vec::new(),
            });

            let settlement_id = world.settlements.get_id();
            let pop_id = world.pops.get_id();

            let pop = world.pops.insert(Pop {
                id: pop_id.clone(),
                size: 100,
                farmed_good: Some(Wheat),
                culture: culture_id.clone(),
                settlement: settlement_id.clone(),
                coordinate,
                satiety: Satiety {
                    base: 0.0,
                    luxury: 0.0,
                },
                kid_buffer: KidBuffer::new(),
                owned_goods: GoodStorage(HashMap::new()),
            });

            pop.upgrade().unwrap().borrow_mut().owned_goods.add(Wheat, 30000.0);

            world.insert_settlement(Settlement {
                id: settlement_id.clone(),
                name: "Test Town".to_owned(),
                pops: vec![pop_id.clone()],
                features: Vec::new(),
                primary_culture: culture_id.clone(),
                coordinate,
                level: SettlementLevel::Village,
            });
        }
    }
}


pub fn pops_yearly_growth(world: &World) {
    for pop_ref in world.pops.id_map.values() {
        let pop_rc = pop_ref.upgrade().unwrap();
        println!("pop size: {}", pop_rc.borrow().size);
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
    for province in world.provinces.rcs.iter() {
        if world.date.month() == province.borrow().harvest_month {
            for settlement in province.borrow().settlements.iter() {
                for pop in world.settlements.get_ref(settlement).borrow().pops.iter() {
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
        for pop in world.pops.id_map.keys() {
            world.add_command(Box::new(PopEatCommand(pop.clone())));
        }
    }
}
