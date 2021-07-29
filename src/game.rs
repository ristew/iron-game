use std::{cell::{RefCell, RefMut}, collections::{HashMap, VecDeque}, fmt::Debug, hash::Hash, ops::Deref, rc::{Rc, Weak}, thread::{sleep, sleep_ms}, time::Duration};
use lazy_static::lazy_static;
use crate::probability::*;
use crate::commands::*;
use crate::world::*;
use crate::storage::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coordinate {
    pub x: isize,
    pub y: isize,
}

impl Coordinate {
    pub fn new(x: isize, y: isize) -> Self {
        Self {
            x,
            y
        }
    }
}

pub enum Terrain {
    Plains,
    Hills,
    Mountains,
    Desert,
    Marsh,
    Forest,
}

impl Factored for Terrain {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => Some(match *self {
                Terrain::Plains => Factor::factor(1.0),
                Terrain::Hills => Factor::factor(0.7),
                Terrain::Mountains => Factor::factor(0.2),
                Terrain::Desert => Factor::factor(0.1),
                Terrain::Marsh => Factor::factor(0.5),
                Terrain::Forest => Factor::factor(0.5),
            }),
        }
    }
}

pub enum Climate {
    Tropical,
    Dry,
    Mild,
    Cold,
}

impl Factored for Climate {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => Some(match *self {
                Climate::Tropical => Factor::factor(1.2),
                Climate::Dry => Factor::factor(0.7),
                Climate::Mild => Factor::factor(1.0),
                Climate::Cold => Factor::factor(0.7),
            }),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum GoodType {
    Wheat,
    Barley,
    OliveOil,
    Fish,
    Wine,
    Iron,
    Copper,
    Tin,
    Bronze,
    Silver,
    Gold,
    Lead,
    Salt,
    PurpleDye,
    Marble,
    Wood,
    Textiles,
    LuxuryClothes,
    Slaves, // ?? how to handle
}

pub use GoodType::*;

#[derive(Debug, Copy, Clone)]
pub enum FactorType {
    CarryingCapacity,
}

pub enum FactorOp {
    Add,
    Mul,
}

pub struct Factor {
    amount: f64,
    op: FactorOp,
}

impl Factor {
    pub fn factor(amount: f64) -> Self {
        Self {
            amount,
            op: FactorOp::Mul,
        }
    }

    pub fn bonus(amount: f64) -> Self {
        Self {
            amount,
            op: FactorOp::Add,
        }
    }
}

pub fn apply_maybe_factors(base: f64, factors: Vec<Option<Factor>>) -> f64 {
    let mut bonus = 0.0;
    let mut res = base;
    for factor_opt in factors.iter() {
        if let Some(factor) = factor_opt {
            match factor.op {
                FactorOp::Add => bonus += factor.amount,
                FactorOp::Mul => res *= factor.amount,
            }
        }
    }

    res + bonus
}

pub trait Factored {
    fn factor(&self, factor: FactorType) -> Option<Factor>;
}

lazy_static! {
    pub static ref FOOD_GOODS: Vec<GoodType> = vec![
        Wheat,
        Barley,
        Fish,
        OliveOil,
        Salt,
        Wine,
    ];
}

impl GoodType {
    pub fn base_satiety(&self) -> Satiety {
        match *self {
            Wheat => Satiety { base: 1.0, luxury: 0.1 },
            Barley => Satiety { base: 1.0, luxury: 0.0 },
            OliveOil => Satiety { base: 0.5, luxury: 0.5 },
            Fish => Satiety { base: 0.8, luxury: 0.2 },
            Wine => Satiety { base: 0.1, luxury: 1.0 },
            Salt => Satiety { base: 0.3, luxury: 0.5 },
            _ => Satiety { base: 0.0, luxury: 0.0 },
        }
    }
}

#[iron_data]
pub struct Province {
    pub id: ProvinceId,
    pub settlements: Vec<SettlementId>,
    pub terrain: Terrain,
    pub climate: Climate,
    pub coordinate: Coordinate,
    pub harvest_month: usize,
}

pub enum SettlementFeature {
    Hilltop,
    Riverside,
    Oceanside,
    Harbor,
    Mines(GoodType),
}

impl Factored for SettlementFeature {
    fn factor(&self, factor: FactorType) -> Option<Factor> {
        match factor {
            FactorType::CarryingCapacity => match *self {
                SettlementFeature::Riverside => Some(Factor::factor(1.2)),
                _ => None,
            }
        }
    }
}

pub enum SettlementLevel {
    Hamlet,
    Village,
    Town,
    City,
    Metropolis
}

#[iron_data]
pub struct Settlement {
    pub id: SettlementId,
    pub name: String,
    pub pops: Vec<PopId>,
    pub features: Vec<SettlementFeature>,
    pub primary_culture: CultureId,
    pub coordinate: Coordinate,
    pub level: SettlementLevel,
}

impl Settlement {
    pub fn carrying_capacity(&self, world: &World) -> f64 {
        let province_rc = world.provinces.get_ref(&world.get_province_coordinate(self.coordinate));
        let province = province_rc.borrow();
        let factor = FactorType::CarryingCapacity;
        let mut factors = vec![province.terrain.factor(factor), province.climate.factor(factor)];
        factors.extend(self.features.iter().map(|f| f.factor(factor)));
        apply_maybe_factors(500.0, factors)
    }
}

#[derive(Debug)]
pub struct KidBuffer(VecDeque<isize>);

impl KidBuffer {
    pub fn new() -> Self {
        Self(VecDeque::new())
    }
    pub fn spawn(&mut self, babies: isize) -> isize {
        println!("spawn babies {}", babies);
        self.0.push_front(babies);
        println!("{:?}", self);
        if self.0.len() > 12 {
            self.0.pop_back().unwrap()
        } else {
            babies / 2
        }
    }

    pub fn starve(&mut self) {
        let cohort = sample(3.0).abs().min(12.0) as usize;
        if self.0.len() > cohort {
            let cohort_size = self.0[cohort];
            let dead_kids = positive_isample(cohort_size / 20 + 1, cohort_size / 10 + 1);
            // println!("cohort {}, size {}, dead {}", cohort, cohort_size, dead_kids);
            self.0[cohort] = (cohort_size - dead_kids).max(0)
        }
    }
}

#[derive(PartialEq)]
pub struct Satiety {
    pub base: f64,
    pub luxury: f64,
}

impl std::ops::Add for Satiety {
    type Output = Satiety;

    fn add(self, rhs: Self) -> Self::Output {
        Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        }
    }
}

impl std::ops::AddAssign for Satiety {
    fn add_assign(&mut self, rhs: Self) {
        *self = Satiety {
            base: self.base + rhs.base,
            luxury: self.luxury + rhs.luxury,
        };
    }
}

impl std::ops::Mul<Satiety> for f64 {
    type Output = Satiety;

    fn mul(self, rhs: Satiety) -> Self::Output {
        Satiety {
            base: rhs.base * self,
            luxury: rhs.luxury * self,
        }
    }
}

pub struct GoodStorage(HashMap<GoodType, f64>);

impl GoodStorage {
    pub fn amount(&self, good: GoodType) -> f64 {
        *self.0.get(&good).unwrap_or(&0.0)
    }

    pub fn consume(&mut self, good: GoodType, amount: f64) -> Option<f64> {
        if let Some(mut stored) = self.0.get_mut(&good) {
            if *stored < amount {
                let deficit = amount - *stored;
                *stored = 0.0;
                Some(deficit)
            } else {
                *stored -= amount;
                None
            }
        } else {
            Some(amount)
        }
    }

    pub fn add(&mut self, good: GoodType, amount: f64) -> f64 {
        if let Some(mut stored) = self.0.get_mut(&good) {
            *stored += amount;
            *stored
        } else {
            self.0.insert(good, amount);
            amount
        }
    }

    pub fn set(&mut self, good: GoodType, amount: f64) {
        self.0.insert(good, amount);
    }

    // pub fn try_eat_diet(&self, diet: Diet) -> Vec<(GoodType, f64)> {
    //     let mut bad_res = Vec::new();

    //     for part in diet.0.iter() {
    //         if self.amount(part.0) < part.1 {
    //             bad_res.push(*part);
    //         }
    //     }

    //     bad_res
    // }
    //
}

pub struct Diet {
    goods: Vec<(GoodType, f64)>,
    satiety: f64,
}

impl Diet {
}

#[iron_data]
pub struct Pop {
    pub id: PopId,
    pub size: isize,
    pub culture: CultureId,
    pub settlement: SettlementId,
    pub coordinate: Coordinate,
    pub kid_buffer: KidBuffer,
    pub owned_goods: GoodStorage,
    pub satiety: Satiety,
    pub farmed_good: Option<GoodType>,
}

impl Pop {
    pub fn good_satiety(&self, good: GoodType) -> Satiety {
        good.base_satiety()
    }

    pub fn die(&mut self, amount: isize) {
        println!("die pops: {}", amount);
        self.size = (self.size - amount).max(0);
        println!("size: {}", self.size);
        if self.size == 0 {
            std::process::exit(0);
        }
    }
}

pub struct PopEatCommand(PopId);

impl Command for PopEatCommand {
    fn run(&self, world: &mut World) {
        let pop = world.pops.get_ref(&self.0);
        let mut total_satiety = Satiety {
            base: 0.0,
            luxury: 0.0,
        };
        let pop_size = pop.borrow().size;
        let target_base = 23.0;
        while total_satiety.base < target_base {
            let mut added = 0.0;
            for good in FOOD_GOODS.iter().rev() {
                let mut amt = pop_size as f64;
                if let Some(deficit) = pop.borrow_mut().owned_goods.consume(*good, amt) {
                    amt -= deficit;
                }
                added += amt;
                total_satiety = total_satiety + (amt / pop_size as f64) * pop.borrow().good_satiety(*good);

                if total_satiety.base > target_base {
                    break;
                }
            }
            if added < 0.01 {
                break;
            }
        }

        if total_satiety.base < 20.0 {
            pop.borrow_mut().kid_buffer.starve();
            if pop.borrow().satiety.base < 10.0 {
                pop.borrow_mut().kid_buffer.starve();
                pop.borrow_mut().die(positive_isample(1 + pop_size / 40, 2 + pop_size / 20))
            }
        }

        pop.borrow_mut().satiety = total_satiety;
    }
}

pub fn harvest(pop: &PopId, world: &World) {
    let pop_rc = world.pops.get_ref(&pop);
    let pop = pop_rc.borrow();
    println!("harvest pop?");
    if let Some(farmed_good) = pop.farmed_good {
        let mut farmed_amount = pop.size as f64;
        let carrying_capacity = world.settlements.get_ref(&pop.settlement).borrow().carrying_capacity(world);
        if farmed_amount > carrying_capacity {
            farmed_amount = carrying_capacity + (farmed_amount - carrying_capacity).sqrt();
        }
        farmed_amount *= 350.0;
        world.add_command(Box::new(SetGoodsCommand {
            good_type: farmed_good,
            amount: farmed_amount,
            pop: pop.id.clone(),
        }));
    }
}

pub enum CultureFeature {
    Warrior,
    Seafaring,
}

#[iron_data]
pub struct Culture {
    pub id: CultureId,
    pub name: String,
    pub religion: ReligionId,
    pub features: Vec<CultureFeature>,
}

#[iron_data]
pub struct Religion {
    pub id: ReligionId,
    pub name: String,
}

pub enum Technology {
    Farming,
}

pub struct TechLevel(usize);

impl Debug for TechLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("#name_id({})", self.0).as_str())
    }
}

impl Hash for TechLevel {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

pub trait IronId {
    type Target;
    fn try_borrow(&self) -> Option<Rc<RefCell<Self::Target>>>;
    fn set_reference(&self, reference: Rc<RefCell<Self::Target>>);
    fn new(id: usize) -> Self;
}


pub trait IronData {
    type DataType;
    type IdType: Eq + Hash;

    fn id(&self) -> Self::IdType;
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
    for i in 0..5 {
        for j in 0..5 {
            let province_id = world.provinces.get_id();
            let coordinate = Coordinate::new(i, j);
            world.insert_province(Province {
                id: province_id,
                settlements: Vec::new(),
                terrain: Terrain::Hills,
                climate: Climate::Mild,
                coordinate,
                harvest_month: 8,
            });

            let settlement_id = world.settlements.get_id();
            let pop_id = world.pops.get_id();

            let pop = world.pops.insert(Pop {
                id: pop_id.clone(),
                size: 100,
                culture: culture_id.clone(),
                settlement: settlement_id.clone(),
                coordinate,
                kid_buffer: KidBuffer::new(),
                owned_goods: GoodStorage(HashMap::new()),
                satiety: Satiety {
                    base: 0.0,
                    luxury: 0.0,
                },
                farmed_good: Some(Wheat),
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

pub fn game_loop() {
    let mut world: World = Default::default();

    create_test_world(&mut world);

    loop {
        world.date.day += 1;
        day_tick(&world);

        if world.date.is_month() {
            println!("{:?}", world.date);
        }
        world.process_command_queue();
        sleep(Duration::from_millis(2));
    }
}
