use std::{cell::RefCell, collections::HashMap, fmt::Debug, rc::Rc};

use crate::game::*;
use crate::{commands::*, storage::*};

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
    pub commands: Rc<RefCell<Vec<Box<dyn Command>>>>
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
}

impl Default for World {
    fn default() -> Self {
        Self {
            date: Date { day: 0 },
            provinces: Default::default(),
            province_coord_map: Default::default(),
            cultures: Default::default(),
            religions: Default::default(),
            settlements: Default::default(),
            pops: Default::default(),
            commands: Rc::new(RefCell::new(Vec::new())),
        }
    }
}
